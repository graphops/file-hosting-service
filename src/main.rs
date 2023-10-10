use clap::{Command, Subcommand, Arg, Parser};
use config::{Cli, Role};
use std::fs;

mod config;
mod types;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Subfile {
    magnet_link: String,
    file_type: String,
    identifier: String,
    block_range: BlockRange,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct BlockRange {
    start_block: Option<u64>,
    end_block: Option<u64>,
}

fn main() {
    let cli: Cli = Cli::parse();

    match cli.role {
        Role::Leecher(leecher) => {
            tracing::info!(leecher = tracing::field::debug(&leecher), "Leecher request");
        },
        Role::Seeder(seeder) => {
            tracing::info!(seeder = tracing::field::debug(&seeder), "Seeder request");
        },
        Role::Tracker(tracker) => {
            tracing::info!(tracker = tracing::field::debug(&tracker), "Tracker request");
        },
    }
}

fn request_file(ipfs_hash: &str) {
    // Grab subfile.yaml from IPFS
    let subfile_content = get_from_ipfs(ipfs_hash);
    let subfile: Subfile = serde_yaml::from_str(&subfile_content).unwrap();

    // Request torrent tracker and download
    // ...

    // Verify the file
    // ...
}

fn server_config(file_config_path: &str) {
    // Read file configurations
    let file_config_content = fs::read_to_string(file_config_path).unwrap();
    let file_configs: Vec<Subfile> = serde_yaml::from_str(&file_config_content).unwrap();

    for config in file_configs {
        // Generate magnet link, subfile.yaml, and upload to IPFS
        // ...
    }

    // Start seeding
    // ...
}

fn get_from_ipfs(ipfs_hash: &str) -> String {
    // Placeholder function to simulate getting data from IPFS
    String::from("...") // Replace with actual IPFS fetch logic
}
