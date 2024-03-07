use bytes::Bytes;
use object_store::local::LocalFileSystem;
use object_store::{path::Path, ObjectStore};
use object_store::{ObjectMeta, PutResult};
use tokio::io::AsyncWriteExt;

use std::fs::{self, File};
use std::ops::Range;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::{LocalDirectory, ObjectStoreArgs, StorageMethod};
use crate::manifest::{verify_chunk, Error, FileManifestMeta, LocalBundle};

use super::file_hasher::hash_chunk;
use super::remote_object_store::s3_store;
use super::FileManifest;

#[derive(Debug, Clone)]
pub struct Store {
    store: Arc<Box<dyn ObjectStore>>,
    pub storage_method: StorageMethod,
    pub read_concurrency: usize,
    pub write_concurrency: usize,
}

impl Store {
    pub fn new(output_dir: &str) -> Result<Self, Error> {
        let path =
            PathBuf::from_str(output_dir).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        if !path.exists() || !path.is_dir() {
            tracing::debug!("Store path doesn't exist or is not a directory, creating a directory at configured path");
            fs::create_dir_all(&path).map_err(|e| {
                Error::InvalidConfig(format!(
                    "Unable to create local filesystem directory structure for object store: {:?}",
                    e.to_string()
                ))
            })?
        }
        // As long as the provided path is correct, the following should never panic
        Ok(Store {
            store: Arc::new(Box::new(
                LocalFileSystem::new_with_prefix(path).map_err(Error::ObjectStoreError)?,
            )),
            storage_method: StorageMethod::LocalFiles(LocalDirectory {
                output_dir: output_dir.to_string(),
            }),
            //TODO: Make configurable
            read_concurrency: 16,
            write_concurrency: 8,
        })
    }

    pub fn new_with_object(store_config: &ObjectStoreArgs) -> Result<Self, Error> {
        let (store, _) = s3_store(
            &("s3://".to_string() + &store_config.bucket),
            &store_config.region,
            &store_config.endpoint,
            &store_config.bucket,
            &store_config.access_key_id,
            &store_config.secret_key,
        )?;
        // As long as the provided path is correct, the following should never panic
        Ok(Store {
            store,
            storage_method: StorageMethod::ObjectStorage(store_config.clone()),
            read_concurrency: 16,
            write_concurrency: 8,
        })
    }
    /// List out all files in the path, optionally filtered by a prefix to the filesystem
    pub async fn list(&self, prefix: Option<&Path>) -> Result<Vec<ObjectMeta>, Error> {
        Ok(self
            .store
            .list_with_delimiter(prefix)
            .await
            .map_err(Error::ObjectStoreError)?
            .objects)
    }

    /// Find a specific object by file name with optional prefix
    pub async fn find_object(&self, file_name: &str, prefix: Option<&Path>) -> Option<ObjectMeta> {
        let listed = self.list(prefix).await.unwrap();
        listed
            .iter()
            .find(|obj| obj.location.to_string() == file_name)
            .cloned()
    }

    pub async fn range_read(&self, file_name: &str, range: &Range<usize>) -> Result<Bytes, Error> {
        Ok(self
            .store
            .get_range(&Path::from(file_name), range.to_owned())
            .await
            .unwrap())
    }

    pub async fn read(&self, location: &str) -> Result<File, Error> {
        let result = match self.store.get(&Path::from(location)).await.unwrap().payload {
            object_store::GetResultPayload::File(f, _p) => f,
            object_store::GetResultPayload::Stream(_) => {
                return Err(Error::DataUnavailable(
                    "Currently data streams are not supported".to_string(),
                ))
            }
        };

        Ok(result)
    }

    pub async fn multipart_read(
        &self,
        file_name: &str,
        file_path: Option<&Path>,
        chunk_size: Option<usize>,
    ) -> Result<Vec<Bytes>, Error> {
        let object_meta =
            self.find_object(file_name, file_path)
                .await
                .ok_or(Error::DataUnavailable(format!(
                    "Did not find object {:?}",
                    file_path,
                )))?;
        let step = chunk_size.unwrap_or({
            let s = object_meta.size / self.read_concurrency;
            if s > 0 {
                s
            } else {
                object_meta.size
            }
        });
        let ranges = (0..(object_meta.size / step + 1))
            .map(|i| std::ops::Range::<usize> {
                start: i * step,
                end: ((i + 1) * step).min(object_meta.size),
            })
            .collect::<Vec<std::ops::Range<usize>>>();
        let location: Path = if let Some(prefix) = file_path {
            prefix.child(file_name)
        } else {
            Path::from(file_name)
        };
        let result = self
            .store
            .get_ranges(&location, ranges.as_slice())
            .await
            .unwrap();

        Ok(result)
    }

    /// Async write with concurrent uploads at a location path
    pub async fn multipart_write(
        &self,
        location: &str,
        bytes: &[u8],
        chunk_size: Option<usize>,
    ) -> Result<String, Error> {
        let (write_id, mut write) = self
            .store
            .put_multipart(&Path::from(location))
            .await
            .unwrap();
        let size = bytes.len();
        let step = chunk_size.unwrap_or({
            let s = size / self.write_concurrency;
            if s > 0 {
                s
            } else {
                size
            }
        });

        for i in 0..(size / step + 1) {
            let buf = &bytes[i * step..((i + 1) * step).min(size)];
            write.write_all(buf).await.unwrap();
        }
        write.flush().await.unwrap();
        write.shutdown().await.unwrap();
        drop(write);
        Ok(write_id)
    }

    /// Single write at a location path
    pub async fn write(&self, location: &str, bytes: &[u8]) -> Result<PutResult, Error> {
        self.store
            .put(&Path::from(location), bytes.to_vec().into())
            .await
            .map_err(Error::ObjectStoreError)
    }

    /// Delete the file if exists
    pub async fn delete(&self, location: &str) -> Result<(), Error> {
        self.store
            .delete(&Path::from(location))
            .await
            .map_err(Error::ObjectStoreError)
    }

    pub async fn file_manifest(
        &self,
        file_name: &str,
        prefix: Option<&Path>,
        chunk_size: Option<usize>,
    ) -> Result<FileManifest, Error> {
        let parts = self.multipart_read(file_name, prefix, chunk_size).await?;
        let total_bytes = parts.iter().map(|b| b.len() as u64).sum();
        let byte_size_used = parts
            .first()
            .ok_or(Error::ChunkInvalid(format!(
                "No chunk produced from object store {} with prefix {:#?}, with chunk size config of {:#?}",
                file_name, prefix, chunk_size
            )))?
            .len();
        let chunk_hashes = parts.iter().map(|c| hash_chunk(c)).collect();

        Ok(FileManifest {
            total_bytes,
            chunk_size: byte_size_used as u64,
            chunk_hashes,
        })
    }

    /// Validate the local files against a given bundle specification
    pub async fn validate_local_bundle(&self, local: &LocalBundle) -> Result<&Self, Error> {
        tracing::trace!(
            bundle = tracing::field::debug(&local),
            "Read and verify bundle. This may cause a long initialization time."
        );

        // Read all files in bundle to verify locally. This may cause a long initialization time
        //TODO: allow for concurrent validation of files
        for file_meta in &local.bundle.file_manifests {
            self.read_and_validate_file(file_meta, &local.local_path)
                .await?;
        }

        tracing::trace!("Successfully verified the local serving files");
        Ok(self)
    }

    /// Read and validate file
    pub async fn read_and_validate_file(
        &self,
        file: &FileManifestMeta,
        prefix: &Path,
    ) -> Result<(), Error> {
        // read file by file_manifest.file_name
        let meta_info = &file.meta_info;
        let file_manifest = &file.file_manifest;
        // let mut file_path = self.local_path.clone();
        // file_path.push(meta_info.name.clone());
        tracing::trace!(
            // file_path = tracing::field::debug(&file_path),
            file_prefix = tracing::field::debug(&prefix),
            file_manifest = tracing::field::debug(&file_manifest),
            "Verify file"
        );

        // loop through file manifest byte range
        //multipart read/ vectorized read
        for i in 0..(file_manifest.total_bytes / file_manifest.chunk_size + 1) {
            // read range
            let start = i * file_manifest.chunk_size;
            let end: usize =
                (u64::min(start + file_manifest.chunk_size, file_manifest.total_bytes) - 1)
                    .try_into()
                    .unwrap();
            tracing::trace!(
                i,
                start_byte = tracing::field::debug(&start),
                end_byte = tracing::field::debug(&end),
                "Verify chunk index"
            );
            let chunk_hash = file_manifest.chunk_hashes[i as usize].clone();

            // read chunk
            // let chunk_data = read_chunk(&file_path, (start, end))?;
            let start: usize = start.try_into().unwrap();
            // let length: usize = end - start + 1;
            let range = std::ops::Range {
                start,
                end: end + 1,
            };
            let file_name = meta_info.name.clone();
            let chunk_data = self.range_read(&file_name, &range).await?;
            // verify chunk
            if !verify_chunk(&chunk_data, &chunk_hash) {
                tracing::error!(
                    file = tracing::field::debug(&file_name),
                    chunk_index = tracing::field::debug(&i),
                    chunk_hash = tracing::field::debug(&chunk_hash),
                    "Cannot locally verify the serving file"
                );
                return Err(Error::InvalidConfig(format!(
                    "Failed to validate the local version of file {}",
                    meta_info.hash
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::{distributions::DistString, thread_rng};

    use crate::{
        manifest::store::*,
        test_util::{create_random_temp_file, simple_bundle, CHUNK_SIZE},
    };

    #[tokio::test]
    async fn test_local_list() {
        let file_size = CHUNK_SIZE;
        let (temp_file, temp_path) = create_random_temp_file(file_size as usize).unwrap();

        let path = std::path::Path::new(&temp_path);
        let readdir = path.parent().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        let object_store = Store::new(readdir).unwrap();
        let res = object_store.list(None).await.unwrap();
        let found_obj = res
            .iter()
            .find(|obj| obj.location.to_string() == file_name)
            .unwrap();
        assert!(found_obj.size == file_size as usize);

        drop(temp_file);
    }

    #[tokio::test]
    async fn test_local_rw() {
        // Create random files
        let file_size = CHUNK_SIZE * 25;
        let mut rng = thread_rng();
        let test_string = rand::distributions::Alphanumeric
            .sample_string(&mut rng, file_size.try_into().unwrap());

        let directory_pathbuf = std::env::current_dir().unwrap();
        let directory = directory_pathbuf.to_str().unwrap();

        // Write with adjusted concurrency
        let object_store = Store::new(directory).unwrap();
        let new_file_name = "tempfile";
        let test_bytes = test_string.as_bytes();
        let write_res = object_store
            .multipart_write(new_file_name, test_bytes, None)
            .await;
        assert!(write_res.is_ok());

        // Read with fixed concurrency
        let read_res: Vec<Bytes> = object_store
            .multipart_read(new_file_name, None, Some(CHUNK_SIZE.try_into().unwrap()))
            .await
            .unwrap();
        let flattened_vec: Vec<u8> = read_res.into_iter().flat_map(|b| b.to_vec()).collect();
        assert!(flattened_vec.as_slice() == test_bytes);

        // Write with fixed concurrency
        let object_store = Store::new(directory).unwrap();
        let new_file_name = "tempfile";
        let test_bytes = test_string.as_bytes();
        let write_res = object_store
            .multipart_write(new_file_name, test_bytes, None)
            .await;
        assert!(write_res.is_ok());

        // Read with adjusted concurrency
        let read_res: Vec<Bytes> = object_store
            .multipart_read(new_file_name, None, None)
            .await
            .unwrap();
        let flattened_vec: Vec<u8> = read_res.into_iter().flat_map(|b| b.to_vec()).collect();
        assert!(flattened_vec.as_slice() == test_bytes);

        // Delete
        assert!(object_store.delete(new_file_name).await.is_ok());
    }

    #[tokio::test]
    async fn test_find_example_file() {
        let main_directory = "../example-file";
        let file_prefix = Path::from("");
        let file_name = "example-create-17686085.dbin";
        let store = Store::new(main_directory).unwrap();
        let metadata = store.find_object(file_name, Some(&file_prefix)).await;

        println!("store: {store:?}, metadata: {metadata:?}");
        assert!(metadata.is_some())
    }

    #[tokio::test]
    async fn test_read_and_validate_file() {
        let store = Store::new("../example-file").unwrap();
        let mut bundle = simple_bundle();
        let file_meta = bundle.file_manifests.first().unwrap();
        let path = Path::from("");
        let res = store.read_and_validate_file(file_meta, &path).await;
        assert!(res.is_ok());

        // Add tests for failure cases
        if let Some(file_meta) = bundle.file_manifests.first_mut() {
            if let Some(first_hash) = file_meta.file_manifest.chunk_hashes.first_mut() {
                *first_hash += "1";
            }
        }
        let file_meta = bundle.file_manifests.first().unwrap();
        let res = store.read_and_validate_file(file_meta, &path).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_validate_local_bundle() {
        let mut bundle = simple_bundle();
        let store = Store::new("../example-file").unwrap();
        let _local_path = Path::from("");
        let local = LocalBundle {
            bundle: bundle.clone(),
            local_path: Path::from(""),
        };
        let res = store.validate_local_bundle(&local).await;

        assert!(res.is_ok());

        // Add tests for failure cases
        if let Some(file_meta) = bundle.file_manifests.first_mut() {
            if let Some(first_hash) = file_meta.file_manifest.chunk_hashes.first_mut() {
                *first_hash += "1";
            }
        }
        let local = LocalBundle {
            bundle,
            local_path: Path::from(""),
        };
        let res = store.validate_local_bundle(&local).await;
        assert!(res.is_err());
    }
}
