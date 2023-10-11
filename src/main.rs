
use config::{Cli, Role};
use ipfs::IpfsClient;
// use ipfs_api_prelude::*;
use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;
// use ipfs_api_backend_hyper::{IpfsClient, IpfsApi};
use dotenv::dotenv;
use std::fs;
use types::Subfile;
use leecher::leech;
use seeder::seed;

mod config;
mod ipfs;
mod types;
mod leecher;
mod seeder;

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
            let client = IpfsClient::localhost();
            // // Create IPFS file
            match seed(&client, &seeder).await {
                Ok(r) => {
                    tracing::info!(result = tracing::field::debug(&r), "Completed seed");
                }
                Err(e) => {
                    tracing::error!(error = tracing::field::debug(&e), "Failed to seed");
                }
            }
        }
        Role::Tracker(tracker) => {
            tracing::info!(tracker = tracing::field::debug(&tracker), "Tracker request");
        }
    }
}

// fn server_config(file_config_path: &str) {
//     // Read file configurations
//     let file_config_content = fs::read_to_string(file_config_path).unwrap();
//     let file_configs: Vec<Subfile> = serde_yaml::from_str(&file_config_content).unwrap();

//     for _config in file_configs {
//         // Generate magnet link, subfile.yaml, and upload to IPFS
//         // ...
//     }

//     // Start seeding
//     // ...
// }
