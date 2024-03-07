use crate::config::PublisherArgs;
use crate::errors::Error;
use crate::manifest::store::Store;
use crate::manifest::{
    ipfs::{AddResponse, IpfsClient},
    BlockRange, BundleManifest, FileMetaInfo,
};
use object_store::path::Path;
use serde_yaml::to_string;

pub struct ManifestPublisher {
    ipfs_client: IpfsClient,
    store: Store,
    config: PublisherArgs,
}

impl ManifestPublisher {
    pub fn new(ipfs_client: IpfsClient, config: PublisherArgs) -> Self {
        let store = Store::new(&config.storage_method).expect("Create store");

        ManifestPublisher {
            ipfs_client,
            store,
            config,
        }
    }

    /// Takes file_path, create file_manifest, build merkle tree, publish, write to output
    pub async fn hash_and_publish_file(
        &self,
        file_name: &str,
        file_prefix: Option<&Path>,
    ) -> Result<AddResponse, Error> {
        let yaml_str = self.write_file_manifest(file_name, file_prefix).await?;

        let added: AddResponse = self
            .ipfs_client
            .add(yaml_str.as_bytes().to_vec())
            .await
            .map_err(Error::IPFSError)?;
        tracing::debug!(
            added = tracing::field::debug(&added),
            "Added yaml file to IPFS"
        );

        Ok(added)
    }

    pub async fn hash_and_publish_files(&self) -> Result<Vec<FileMetaInfo>, Error> {
        let mut root_hashes = Vec::new();

        let file_names = &self.config.file_names;
        tracing::trace!(
            file_names = tracing::field::debug(&file_names),
            "hash_and_publish_files",
        );

        for file_name in file_names {
            let ipfs_hash = self.hash_and_publish_file(file_name, None).await?.hash;
            root_hashes.push(FileMetaInfo {
                name: file_name.to_string(),
                hash: ipfs_hash,
            });
        }

        Ok(root_hashes)
    }

    pub fn construct_bundle_manifest(
        &self,
        file_meta_info: Vec<FileMetaInfo>,
    ) -> Result<String, Error> {
        let manifest = BundleManifest {
            files: file_meta_info,
            file_type: self.config.file_type.clone(),
            spec_version: self.config.bundle_version.clone(),
            description: self.config.description.clone(),
            chain_id: self.config.chain_id.clone(),
            block_range: BlockRange {
                start_block: self.config.start_block,
                end_block: self.config.end_block,
            },
        };
        let yaml = serde_yaml::to_string(&manifest).map_err(Error::YamlError)?;
        Ok(yaml)
    }

    pub async fn publish_bundle_manifest(&self, manifest_yaml: &str) -> Result<String, Error> {
        let ipfs_hash = self
            .ipfs_client
            .add(manifest_yaml.as_bytes().to_vec())
            .await
            .map_err(Error::IPFSError)?
            .hash;

        Ok(ipfs_hash)
    }

    pub async fn publish(&self) -> Result<String, Error> {
        let meta_info = self.hash_and_publish_files().await?;

        tracing::trace!(
            meta_info = tracing::field::debug(&meta_info),
            "hash_and_publish_files",
        );
        match self.construct_bundle_manifest(meta_info) {
            Ok(manifest_yaml) => {
                let ipfs_hash = self.publish_bundle_manifest(&manifest_yaml).await?;
                tracing::info!(
                    "Published bundle manifest to IPFS with hash: {}",
                    &ipfs_hash
                );
                Ok(ipfs_hash)
            }
            Err(e) => Err(e),
        }
    }

    // pub async fn object_store_write_file_manifest(&self, file_name: &str) -> Result<String, Error> {
    pub async fn write_file_manifest(
        &self,
        file_name: &str,
        file_prefix: Option<&Path>,
    ) -> Result<String, Error> {
        let file_manifest = self
            .store
            .file_manifest(
                file_name,
                file_prefix,
                Some(self.config.chunk_size as usize),
            )
            .await?;

        tracing::trace!(
            file = tracing::field::debug(&file_manifest),
            "Created file manifest"
        );

        let yaml = to_string(&file_manifest).map_err(Error::YamlError)?;
        Ok(yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LocalDirectory, StorageMethod};

    #[tokio::test]
    async fn test_write_file_manifest() {
        let client = IpfsClient::localhost();
        let args = PublisherArgs {
            storage_method: StorageMethod::LocalFiles(LocalDirectory {
                main_dir: String::from("../example-file"),
            }),
            chunk_size: 1048576,
            ..Default::default()
        };
        let publisher = ManifestPublisher::new(client, args);
        let name = "example-create-17686085.dbin";

        // Hash and publish a single file
        let file_manifest_yaml = publisher.write_file_manifest(name, None).await;

        assert!(file_manifest_yaml.is_ok());
    }

    #[tokio::test]
    #[ignore] // Run when there is a localhost IPFS node
    async fn test_publish() {
        let client = IpfsClient::localhost();
        let args = PublisherArgs {
            storage_method: StorageMethod::LocalFiles(LocalDirectory {
                main_dir: String::from("../example-file"),
            }),
            ..Default::default()
        };
        let builder = ManifestPublisher::new(client, args);
        let name = "example-create-17686085.dbin";

        // Hash and publish a single file
        let hash = builder
            .hash_and_publish_file(name, None)
            .await
            .unwrap()
            .hash;

        // Construct and publish a bundle manifest
        let meta_info = vec![FileMetaInfo {
            name: name.to_string(),
            hash,
        }];

        if let Ok(manifest_yaml) = builder.construct_bundle_manifest(meta_info) {
            if let Ok(ipfs_hash) = builder.publish_bundle_manifest(&manifest_yaml).await {
                tracing::info!("Published bundle manifest to IPFS with hash: {}", ipfs_hash);
            }
        }
    }
}
