use crate::ipfs::IpfsClient;
use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use crate::types::Subfile;

// Fetch subfile yaml from IPFS
async fn fetch_subfile_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<Subfile, anyhow::Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(30);

    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // add jitter to delays
        .take(5); // limit to 5 retries

    let file_bytes = Retry::spawn(retry_strategy, || client.cat_all(ipfs_hash, timeout)).await?;

    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::info!("Got yaml file content");

    let subfile = convert_to_subfile(content)?;

    Ok(subfile)
}

fn convert_to_subfile(value: serde_yaml::Value) -> Result<Subfile, anyhow::Error> {
    tracing::trace!(
        value = tracing::field::debug(&value),
        "Parse yaml value into a subfile"
    );
    let subfile: Subfile = serde_yaml::from_value(value)?;

    //TODO: verify that the magnet link will truly result in the target subfile
    Ok(subfile)
}

pub async fn leech(client: &IpfsClient, ipfs_hash: &str) -> Result<Subfile, anyhow::Error> {
    let subfile: Subfile = fetch_subfile_from_ipfs(client, ipfs_hash).await?;

    //TODO: Request torrent tracker and download

    //TODO: Verify the file

    Ok(subfile)
}
