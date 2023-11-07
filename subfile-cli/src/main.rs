use dotenv::dotenv;

use subfile_cli::{
    config::{Cli, Role},
    ipfs::IpfsClient,
    publisher::SubfilePublisher,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli: Cli = Cli::args();

    tracing::info!(cli = tracing::field::debug(&cli), "Running cli");

    let client = if let Ok(client) = IpfsClient::new(&cli.ipfs_gateway) {
        client
    } else {
        IpfsClient::localhost()
    };

    match cli.role {
        Role::Downloader(leecher) => {
            tracing::info!(
                leecher = tracing::field::debug(&leecher),
                "Downloader request"
            );
            // Create IPFS client
            let _client = if let Ok(client) = IpfsClient::new(&cli.ipfs_gateway) {
                client
            } else {
                IpfsClient::localhost()
            };
            // Use client to grab IPFS

            // Parse IPFS

            // Validate IPFS against extra input to make sure it is the target file

            // Grab the magnet link inside IPFS file and start torrent leeching
            // match leech(&client, &leecher.ipfs_hash, &leecher.output_dir).await {
            //     Ok(r) => {
            //         tracing::info!(
            //             result = tracing::field::debug(&r),
            //             "End of leeching"
            //         );
            //     }
            //     Err(e) => {
            //         tracing::error!(error = tracing::field::debug(&e), "Failed to leech");
            //     }
            // }
        }
        Role::Publisher(config) => {
            tracing::info!(config = tracing::field::debug(&config), "Publisher request");

            let publisher = SubfilePublisher::new(client);

            match publisher.publish(&config.file_path).await {
                Ok(r) => {
                    tracing::info!(result = tracing::field::debug(&r), "Published");
                }
                Err(e) => {
                    tracing::error!(error = tracing::field::debug(&e), "Failed to build");
                }
            }
        }
        Role::Server(server) => {
            tracing::info!(server = tracing::field::debug(&server), "Tracker request");
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
