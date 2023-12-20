use bytes::Bytes;
use futures::StreamExt;
use object_store::{path::Path, ObjectStore};
use object_store::{ObjectMeta, PutResult};

use object_store::local::LocalFileSystem;
use tokio::io::AsyncWriteExt;

use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::subfile::Error;

use super::file_hasher::hash_chunk;
use super::ChunkFile;

pub struct Store {
    local_file_system: Arc<LocalFileSystem>,
    read_concurrency: usize,
    write_concurrency: usize,
}

impl Store {
    pub fn new(path: &str) -> Result<Self, Error> {
        let path = PathBuf::from_str(path).map_err(|e| Error::InvalidConfig(e.to_string()))?;
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
            local_file_system: Arc::new(
                LocalFileSystem::new_with_prefix(path).map_err(Error::ObjectStoreError)?,
            ),
            //TODO: Make configurable
            read_concurrency: 16,
            write_concurrency: 8,
        })
    }

    /// List out all files in the path, optionally filtered by a prefix to the filesystem
    pub async fn list(&self, prefix: Option<&Path>) -> Result<Vec<ObjectMeta>, Error> {
        let mut list_stream = self.local_file_system.list(prefix);

        let mut objects = vec![];
        while let Ok(Some(meta)) = list_stream.next().await.transpose() {
            tracing::trace!("File name: {}, size: {}", meta.location, meta.size);
            objects.push(meta.clone());
        }
        Ok(objects)
    }

    /// Find a specific object by file name with optional prefix
    pub async fn find_object(&self, file_name: &str, prefix: Option<&Path>) -> Option<ObjectMeta> {
        let listed = self.list(prefix).await.unwrap();
        listed
            .iter()
            .find(|obj| obj.location.to_string() == file_name)
            .cloned()
    }

    pub async fn range_read(&self, file_name: &str, range: Range<usize>) -> Result<Bytes, Error> {
        Ok(self
            .local_file_system
            .get_range(&Path::from(file_name), range)
            .await
            .unwrap())
    }

    pub async fn multipart_read(
        &self,
        location: &str,
        chunk_size: Option<usize>,
    ) -> Result<Vec<Bytes>, Error> {
        let object_meta = self
            .find_object(location, None)
            .await
            .ok_or(Error::DataUnavilable(format!(
                "Did not find file {}",
                location
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

        let result = self
            .local_file_system
            .get_ranges(&Path::from(location), ranges.as_slice())
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
            .local_file_system
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
        self.local_file_system
            .put(&Path::from(location), bytes.to_vec().into())
            .await
            .map_err(Error::ObjectStoreError)
    }

    /// Delete the file if exists
    pub async fn delete(&self, location: &str) -> Result<(), Error> {
        self.local_file_system
            .delete(&Path::from(location))
            .await
            .map_err(Error::ObjectStoreError)
    }

    pub async fn chunk_file(
        &self,
        location: &str,
        chunk_size: Option<usize>,
    ) -> Result<ChunkFile, Error> {
        let parts = self.multipart_read(location, chunk_size).await?;
        let total_bytes = parts.iter().map(|b| b.len() as u64).sum();
        let byte_size_used = parts
            .first()
            .ok_or(Error::ChunkInvalid(format!(
                "No chunk produced from object store {}, with chunk size config of {:#?}",
                location, chunk_size
            )))?
            .len();
        let chunk_hashes = parts.iter().map(|c| hash_chunk(c)).collect();

        Ok(ChunkFile {
            total_bytes,
            chunk_size: byte_size_used as u64,
            chunk_hashes,
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::{distributions::DistString, thread_rng};

    use crate::{
        subfile::local_file_system::*,
        test_util::{create_random_temp_file, CHUNK_SIZE},
    };

    #[tokio::test]
    async fn test_local_list() {
        let file_size = CHUNK_SIZE * 25;
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
            .multipart_read(new_file_name, Some(CHUNK_SIZE.try_into().unwrap()))
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
            .multipart_read(new_file_name, None)
            .await
            .unwrap();
        let flattened_vec: Vec<u8> = read_res.into_iter().flat_map(|b| b.to_vec()).collect();
        assert!(flattened_vec.as_slice() == test_bytes);

        // Delete
        assert!(object_store.delete(new_file_name).await.is_ok());
    }
}
