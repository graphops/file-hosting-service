use std::{path::PathBuf, time::Duration};

use crate::{
    ipfs::IpfsClient,
    subfile::{ChunkFile, ChunkFileMeta, Subfile, SubfileManifest},
};

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

    tracing::debug!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let subfile = parse_subfile_manifest(content)?;

    tracing::debug!(
        subfile = tracing::field::debug(&subfile),
        "subfile manifest"
    );

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
    tracing::debug!(ipfs_hash, "Fetch chunk file from IPFS");
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(10);

    let file_bytes = client.cat_all(ipfs_hash, timeout).await?;

    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::debug!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let chunk_file = parse_chunk_file(content)?;

    Ok(chunk_file)
}

/// Read subfile from IPFS, build a version relative to local access
pub async fn read_subfile(
    client: &IpfsClient,
    ipfs: &str,
    local_path: PathBuf,
) -> Result<Subfile, anyhow::Error> {
    let manifest = fetch_subfile_from_ipfs(client, ipfs).await?;

    // Get and Parse the YAML file to get chunk hashes
    let mut chunk_files = vec![];
    for file_info in &manifest.files {
        let chunk_file = fetch_chunk_file_from_ipfs(client, &file_info.hash).await?;

        chunk_files.push(ChunkFileMeta {
            meta_info: file_info.clone(),
            chunk_file,
        });
    }

    Ok(Subfile {
        ipfs_hash: ipfs.to_string(),
        local_path,
        manifest,
        chunk_files,
    })
}
