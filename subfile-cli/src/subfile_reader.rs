use std::{error::Error, time::Duration};

use crate::{file_hasher::ChunkFile, ipfs::IpfsClient, publisher::SubfileManifest};

/// Parse yaml into Subfile manifest
pub fn parse_subfile_manifest(yaml: serde_yaml::Value) -> Result<SubfileManifest, anyhow::Error> {
    Ok(serde_yaml::from_value(yaml)?)
}

// Fetch subfile yaml from IPFS
// async fn fetch_subfile_from_ipfs(client: &IpfsClient, ipfs_hash: &str) -> Result<serde_yaml::Mapping, anyhow::Error> {
pub async fn fetch_subfile_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<SubfileManifest, anyhow::Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(10);

    let file_bytes = client.cat_all(ipfs_hash, timeout).await?;

    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::info!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let subfile = parse_subfile_manifest(content)?;

    Ok(subfile)
}

/// Parse yaml into a chunk file
pub fn parse_chunk_file(yaml: serde_yaml::Value) -> Result<ChunkFile, anyhow::Error> {
    Ok(serde_yaml::from_value(yaml)?)
}

// Fetch subfile yaml from IPFS
// async fn fetch_subfile_from_ipfs(client: &IpfsClient, ipfs_hash: &str) -> Result<serde_yaml::Mapping, anyhow::Error> {
pub async fn fetch_chunk_file_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<ChunkFile, anyhow::Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(10);

    let file_bytes = client.cat_all(ipfs_hash, timeout).await?;

    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::info!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let chunk_file = parse_chunk_file(content)?;

    Ok(chunk_file)
}

pub async fn read_manifest(client: &IpfsClient, ipfs: &str) -> Result<(), Box<dyn Error>> {
    let manifest = fetch_subfile_from_ipfs(client, ipfs).await?;

    for file in manifest.files {
        let chunk_file = fetch_chunk_file_from_ipfs(client, &file.hash).await?;
        println!("Chunk file root: {:?}", chunk_file.merkle_root);
    }

    Ok(())
}
