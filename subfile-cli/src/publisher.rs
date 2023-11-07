use serde::{Deserialize, Serialize};

use crate::file_hasher::write_chunk_file;
use crate::ipfs::{AddResponse, IpfsClient};

#[derive(Serialize, Deserialize)]
pub struct SubfileManifest {
    pub files: Vec<FileMetaInfo>,
    // Add additional metadata as needed
}

#[derive(Serialize, Deserialize)]
pub struct FileMetaInfo {
    pub path: String, // file name instead?
    pub hash: String,
    // Add additional metadata as needed
}

pub struct SubfilePublisher {
    ipfs_client: IpfsClient,
    // Other fields as needed
}

impl SubfilePublisher {
    pub fn new(ipfs_client: IpfsClient) -> Self {
        SubfilePublisher {
            ipfs_client,
            // Initialize other fields
        }
    }

    /// Takes file_path, create chunk_file, build merkle tree, publish, write to output
    pub async fn hash_and_publish_file(
        &self,
        file_path: &str,
    ) -> Result<AddResponse, anyhow::Error> {
        let yaml_str = write_chunk_file(file_path)?;

        let added: AddResponse = self.ipfs_client.add(yaml_str.as_bytes().to_vec()).await?;
        tracing::info!(
            added = tracing::field::debug(&added),
            "Added yaml file to IPFS"
        );

        Ok(added)
    }

    pub async fn hash_and_publish_files(
        &self,
        file_paths: &[&str],
    ) -> Result<Vec<String>, anyhow::Error> {
        let mut root_hashes = Vec::new();

        for &file_path in file_paths {
            let ipfs_hash = self.hash_and_publish_file(file_path).await?.hash;
            root_hashes.push(ipfs_hash);
        }

        Ok(root_hashes)
    }

    pub fn construct_subfile_manifest(
        &self,
        file_meta_info: Vec<FileMetaInfo>,
    ) -> Result<String, serde_yaml::Error> {
        let manifest = SubfileManifest {
            files: file_meta_info,
        };
        let yaml = serde_yaml::to_string(&manifest)?;
        Ok(yaml)
    }

    pub async fn publish_subfile_manifest(
        &self,
        manifest_yaml: &str,
    ) -> Result<String, anyhow::Error> {
        let ipfs_hash = self
            .ipfs_client
            .add(manifest_yaml.as_bytes().to_vec())
            .await?
            .hash;

        Ok(ipfs_hash)
    }

    //TODO: use the full config args for publishing
    pub async fn publish(&self, file_path: &str) -> Result<String, anyhow::Error> {
        let hash = match self.hash_and_publish_file(file_path).await {
            Ok(added) => {
                println!("Published file to IPFS: {:#?}", added);
                added.hash
            }
            Err(e) => return Err(e),
        };

        // Construct and publish a subfile manifest
        let meta_info = vec![FileMetaInfo {
            path: file_path.to_string(),
            hash,
        }];

        match self.construct_subfile_manifest(meta_info) {
            Ok(manifest_yaml) => match self.publish_subfile_manifest(&manifest_yaml).await {
                Ok(ipfs_hash) => {
                    println!(
                        "Published subfile manifest to IPFS with hash: {}",
                        ipfs_hash
                    );
                    Ok(ipfs_hash)
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish() {
        let client = IpfsClient::localhost();

        let builder = SubfilePublisher::new(client);
        let path = "./example-file/example-create-17686085.dbin";
        // let chunks1 = chunk_file(Path::new(&path))?;

        // Hash and publish a single file
        let hash = builder.hash_and_publish_file(path).await.unwrap().hash;
        // if let Ok(added) = builder.hash_and_publish_file(&path).await.unwrap() {
        //     println!("Published file to IPFS with hash: {}", added.hash);
        // }

        // Construct and publish a subfile manifest
        let meta_info = vec![FileMetaInfo {
            path: path.to_string(),
            hash,
        }];

        if let Ok(manifest_yaml) = builder.construct_subfile_manifest(meta_info) {
            if let Ok(ipfs_hash) = builder.publish_subfile_manifest(&manifest_yaml).await {
                println!(
                    "Published subfile manifest to IPFS with hash: {}",
                    ipfs_hash
                );
            }
        }
    }
}