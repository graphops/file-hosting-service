use clap::Parser;
use config::{Cli, Role};
use ipfs::IpfsClient;
// use ipfs_api_prelude::*;
use std::time::Duration;
// use ipfs_api_backend_hyper::{IpfsClient, IpfsApi};
use dotenv::dotenv;
use std::fs;
use types::Subfile;

mod config;
mod ipfs;
mod types;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli: Cli = Cli::args();

    tracing::info!(cli = tracing::field::debug(&cli), "Running cli");

    match cli.role {
        Role::Leecher(leecher) => {
            tracing::info!(leecher = tracing::field::debug(&leecher), "Leecher request");
            // Create IPFS client
            let client = if let Ok(client) = IpfsClient::new(&cli.ipfs_gateway) {
                client
            } else {
                IpfsClient::localhost()
            };
            // Use client to grab IPFS

            // Parse IPFS

            // Validate IPFS against extra input to make sure it is the target file

            // Grab the magnet link inside IPFS file and start torrent leeching

            match leech(&client, &leecher.ipfs_hash).await {
                Ok(r) => {
                    tracing::info!(result = tracing::field::debug(&r), "Completed leech");
                }
                Err(e) => {
                    tracing::error!(error = tracing::field::debug(&e), "Failed to leech");
                }
            }
        }
        Role::Seeder(seeder) => {
            tracing::info!(seeder = tracing::field::debug(&seeder), "Seeder request");
            // let client = IpfsClient::localhost();
            // // Create IPFS file
            // let link = client.add(file.into()).await.unwrap().hash;
        }
        Role::Tracker(tracker) => {
            tracing::info!(tracker = tracing::field::debug(&tracker), "Tracker request");
        }
    }
}

// Fetch subfile yaml from IPFS
// async fn fetch_subfile_from_ipfs(client: &IpfsClient, ipfs_hash: &str) -> Result<serde_yaml::Mapping, anyhow::Error> {
async fn fetch_subfile_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<Subfile, anyhow::Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(30);

    // let file_bytes = client.cat(ipfs_hash, Some(timeout)).await?;
    let file_bytes = client.cat_all(ipfs_hash, timeout).await?;

    // let data = String::from_utf8(file_bytes.to_vec()).unwrap();

    // let yaml = String::from_utf8(file_bytes)?;

    // Parse the content into a Subfile structure
    // let subfile: String = serde_yaml::from_slice(&content)?;
    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::info!(
        content = tracing::field::debug(&content),
        "Read file content"
    );

    let subfile = convert_to_subfile(content)?;

    // d["foo"]["bar"]
    //     .as_str()
    //     .map(|s| s.to_string())
    //     .ok_or(anyhow!("Could not find key foo.bar in something.yaml")

    Ok(subfile)
}

fn convert_to_subfile(value: serde_yaml::Value) -> Result<Subfile, anyhow::Error> {
    let subfile: Subfile = serde_yaml::from_value(value)?;
    Ok(subfile)
}

async fn leech(client: &IpfsClient, ipfs_hash: &str) -> Result<Subfile, anyhow::Error> {
    // Grab subfile.yaml from IPFS
    let subfile: Subfile = fetch_subfile_from_ipfs(client, ipfs_hash).await?;

    // Request torrent tracker and download

    // Verify the file

    Ok(subfile)
}

fn server_config(file_config_path: &str) {
    // Read file configurations
    let file_config_content = fs::read_to_string(file_config_path).unwrap();
    let file_configs: Vec<Subfile> = serde_yaml::from_str(&file_config_content).unwrap();

    for _config in file_configs {
        // Generate magnet link, subfile.yaml, and upload to IPFS
        // ...
    }

    // Start seeding
    // ...
}

fn get_from_ipfs(_ipfs_hash: &str) -> String {
    // Placeholder function to simulate getting data from IPFS
    String::from("...") // Replace with actual IPFS fetch logic
}
