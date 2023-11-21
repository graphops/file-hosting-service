use serde::{Deserialize, Serialize};

use crate::config::PublisherArgs;
use crate::file_hasher::write_chunk_file;
use crate::ipfs::{AddResponse, IpfsClient};
use crate::types::BlockRange;

/// Better mapping of files and chunk files
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubfileManifest {
    pub files: Vec<FileMetaInfo>,
    pub file_type: String,
    pub spec_version: String,
    pub description: String,
    pub chain_id: String,
    pub block_range: BlockRange,
    //TODO: Add additional metadata as needed
    // pub identifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileMetaInfo {
    pub name: String,
    pub hash: String,
    //TODO: Add additional metadata as needed
    // pub file_link: String,
    // pub file_name: String,
    // pub block_range: BlockRange,
}

pub struct SubfilePublisher {
    ipfs_client: IpfsClient,
    config: PublisherArgs,
}

impl SubfilePublisher {
    pub fn new(ipfs_client: IpfsClient, config: PublisherArgs) -> Self {
        SubfilePublisher {
            ipfs_client,
            config,
        }
    }

    /// Takes file_path, create chunk_file, build merkle tree, publish, write to output
    pub async fn hash_and_publish_file(
        &self,
        file_name: &str,
    ) -> Result<AddResponse, anyhow::Error> {
        let yaml_str = write_chunk_file(&self.config.read_dir, file_name)?;

        let added: AddResponse = self.ipfs_client.add(yaml_str.as_bytes().to_vec()).await?;
        tracing::debug!(
            added = tracing::field::debug(&added),
            "Added yaml file to IPFS"
        );

        Ok(added)
    }

    pub async fn hash_and_publish_files(
        &self,
        file_names: &Vec<String>,
    ) -> Result<Vec<FileMetaInfo>, anyhow::Error> {
        let mut root_hashes = Vec::new();

        tracing::trace!(
            file_names = tracing::field::debug(&file_names),
            "hash_and_publish_files",
        );

        for file_name in file_names {
            let ipfs_hash = self.hash_and_publish_file(file_name).await?.hash;
            root_hashes.push(FileMetaInfo {
                name: file_name.to_string(),
                hash: ipfs_hash,
            });
        }

        Ok(root_hashes)
    }

    pub fn construct_subfile_manifest(
        &self,
        file_meta_info: Vec<FileMetaInfo>,
    ) -> Result<String, serde_yaml::Error> {
        let manifest = SubfileManifest {
            files: file_meta_info,
            file_type: self.config.file_type.clone(),
            spec_version: self.config.file_version.clone(),
            description: self.config.description.clone(),
            chain_id: self.config.chain_id.clone(),
            block_range: BlockRange {
                start_block: self.config.start_block,
                end_block: self.config.end_block,
            },
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
    pub async fn publish(&self) -> Result<String, anyhow::Error> {
        let meta_info = match self.hash_and_publish_files(&self.config.file_names).await {
            Ok(added_hashes) => added_hashes,
            Err(e) => return Err(e),
        };

        tracing::trace!(
            meta_info = tracing::field::debug(&meta_info),
            "hash_and_publish_files",
        );
        match self.construct_subfile_manifest(meta_info) {
            Ok(manifest_yaml) => match self.publish_subfile_manifest(&manifest_yaml).await {
                Ok(ipfs_hash) => {
                    tracing::info!(
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
        let args = PublisherArgs {
            ..Default::default()
        };
        let builder = SubfilePublisher::new(client, args);
        let name = "example-create-17686085.dbin";
        // let chunks1 = chunk_file(Path::new(&path))?;

        // Hash and publish a single file
        let hash = builder.hash_and_publish_file(name).await.unwrap().hash;

        // Construct and publish a subfile manifest
        let meta_info = vec![FileMetaInfo {
            name: name.to_string(),
            hash,
        }];

        if let Ok(manifest_yaml) = builder.construct_subfile_manifest(meta_info) {
            if let Ok(ipfs_hash) = builder.publish_subfile_manifest(&manifest_yaml).await {
                tracing::info!(
                    "Published subfile manifest to IPFS with hash: {}",
                    ipfs_hash
                );
            }
        }
    }
}
