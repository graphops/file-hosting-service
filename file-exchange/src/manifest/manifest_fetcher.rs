use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::{
    errors::Error,
    manifest::ipfs::IpfsClient,
    manifest::{Bundle, BundleManifest, FileManifest, FileManifestMeta},
};

/// Parse yaml into Bundle manifest
pub fn parse_bundle_manifest(yaml: serde_yaml::Value) -> Result<BundleManifest, Error> {
    serde_yaml::from_value(yaml).map_err(Error::YamlError)
}

/// Parse yaml to generic T
pub fn parse_yaml<T: DeserializeOwned>(yaml: serde_yaml::Value) -> Result<T, Error> {
    serde_yaml::from_value(yaml).map_err(Error::YamlError)
}

// Fetch bundle yaml from IPFS
pub async fn fetch_bundle_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<BundleManifest, Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(10);

    let file_bytes = client
        .cat_all(ipfs_hash, timeout)
        .await
        .map_err(Error::IPFSError)?;

    let content: serde_yaml::Value = serde_yaml::from_str(
        &String::from_utf8(file_bytes.to_vec()).map_err(|e| Error::ManifestError(e.to_string()))?,
    )
    .map_err(Error::YamlError)?;

    tracing::trace!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let bundle = parse_bundle_manifest(content)?;

    tracing::trace!(bundle = tracing::field::debug(&bundle), "bundle manifest");

    Ok(bundle)
}

/// Parse yaml into a file manifest
pub fn parse_file_manifest(yaml: serde_yaml::Value) -> Result<FileManifest, Error> {
    serde_yaml::from_value(yaml).map_err(Error::YamlError)
}

// Fetch file manifest yaml from IPFS
pub async fn fetch_file_manifest_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<FileManifest, Error> {
    tracing::trace!(ipfs_hash, "Fetch file manifest from IPFS");
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(10);

    let file_bytes = client
        .cat_all(ipfs_hash, timeout)
        .await
        .map_err(Error::IPFSError)?;

    let content: serde_yaml::Value = serde_yaml::from_str(
        &String::from_utf8(file_bytes.to_vec()).map_err(|e| Error::ManifestError(e.to_string()))?,
    )
    .map_err(Error::YamlError)?;

    tracing::trace!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let file_manifest = parse_file_manifest(content)?;

    Ok(file_manifest)
}

/// Read bundle from IPFS, build a version relative to local access
pub async fn read_bundle(client: &IpfsClient, ipfs: &str) -> Result<Bundle, Error> {
    let manifest = fetch_bundle_from_ipfs(client, ipfs).await?;

    // Get and Parse the YAML file to get chunk hashes
    let mut file_manifests = vec![];
    for file_info in &manifest.files {
        let file_manifest = fetch_file_manifest_from_ipfs(client, &file_info.hash).await?;

        file_manifests.push(FileManifestMeta {
            meta_info: file_info.clone(),
            file_manifest,
        });
    }

    Ok(Bundle {
        ipfs_hash: ipfs.to_string(),
        manifest,
        file_manifests,
    })
}
