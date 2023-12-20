use futures::StreamExt;
use object_store::ObjectMeta;
use object_store::{path::Path, ObjectStore};

use object_store::local::LocalFileSystem;

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
}

#[cfg(test)]
mod tests {
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
}
