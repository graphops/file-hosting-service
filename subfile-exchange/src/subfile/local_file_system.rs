use bytes::Bytes;
use futures::StreamExt;
use object_store::ObjectMeta;
use object_store::{path::Path, ObjectStore};

use object_store::local::LocalFileSystem;
use tokio::io::AsyncWriteExt;

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::subfile::Error;

pub struct Store {
    local_file_system: Arc<LocalFileSystem>,
}

impl Store {
    pub fn new(path: &str) -> Result<Self, Error> {
        let path = PathBuf::from_str(path).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        if !path.exists() || !path.is_dir() {
            fs::create_dir_all(&path).map_err(|e| {
                Error::ObjectStoreError(format!(
                    "Unable to create local filesystem directory structure for object store: {:?}",
                    e.to_string()
                ))
            })?
        }
        // As long as the provided path is correct, the following should never panic
        Ok(Store {
            local_file_system: Arc::new(
                LocalFileSystem::new_with_prefix(path)
                    .map_err(|e| Error::ObjectStoreError(e.to_string()))?,
            ),
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

    pub async fn multipart_read(&self, file_name: &str) -> Result<Vec<Bytes>, Error> {
        let object_meta =
            self.find_object(file_name, None)
                .await
                .ok_or(Error::ObjectStoreError(format!(
                    "Did not find file {}",
                    file_name
                )))?;
        let read_concurrency = 16;
        let step = object_meta.size / read_concurrency;
        let ranges = (0..read_concurrency)
            .map(|i| std::ops::Range::<usize> {
                start: i * step,
                end: (i + 1) * step,
            })
            .collect::<Vec<std::ops::Range<usize>>>();

        let result = self
            .local_file_system
            .get_ranges(&Path::from(file_name), ranges.as_slice())
            .await
            .unwrap();

        Ok(result)
    }

    /// Async write with concurrent uploads at a location path
    pub async fn multipart_write(&self, location: &str, bytes: &[u8]) -> Result<String, Error> {
        let (write_id, mut write) = self
            .local_file_system
            .put_multipart(&Path::from(location))
            .await
            .unwrap();
        let size = bytes.len();

        let write_concurrency = 8;
        let step = size / write_concurrency;
        for i in 0..write_concurrency {
            let buf = &bytes[i * step..(i + 1) * step];
            write.write_all(buf).await.unwrap();
        }
        write.flush().await.unwrap();
        write.shutdown().await.unwrap();
        drop(write);
        Ok(write_id)
    }

    /// Delete the file if exists
    pub async fn delete(&self, location: &str) -> Result<(), Error> {
        self.local_file_system
            .delete(&Path::from(location))
            .await
            .map_err(|e| Error::ObjectStoreError(e.to_string()))
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

        // Write
        let object_store = Store::new(directory).unwrap();
        let new_file_name = "tempfile";
        let test_bytes = test_string.as_bytes();
        let write_res = object_store
            .multipart_write(new_file_name, test_bytes)
            .await;
        assert!(write_res.is_ok());

        // Read
        let read_res: Vec<Bytes> = object_store.multipart_read(new_file_name).await.unwrap();
        let flattened_vec: Vec<u8> = read_res.into_iter().flat_map(|b| b.to_vec()).collect();
        assert!(flattened_vec.as_slice() == test_bytes);

        // Delete
        assert!(object_store.delete(new_file_name).await.is_ok());
    }
}
