use subfile_cli::{subfile_client::SubfileDownloader, subfile_server::init_server};

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
        Role::Downloader(config) => {
            tracing::info!(
                config = tracing::field::debug(&config),
                "Downloader request"
            );
            // Create client
            let downloader = SubfileDownloader::new(client, config);
            // Use client to grab IPFS

            // Parse IPFS

            // Validate IPFS against extra input to make sure it is the target file

            // Send range request
            let res = downloader.download_subfile().await;
            tracing::info!("Download result: {:#?}", res);
        }
        Role::Publisher(config) => {
            tracing::info!(config = tracing::field::debug(&config), "Publisher request");

            let publisher = SubfilePublisher::new(client, &config.read_dir);

            match publisher.publish(&config).await {
                Ok(r) => {
                    tracing::info!(result = tracing::field::debug(&r), "Published");
                }
                Err(e) => {
                    tracing::error!(error = tracing::field::debug(&e), "Failed to build");
                }
            }
        }
        Role::Server(server_args) => {
            tracing::info!(
                server = tracing::field::debug(&server_args),
                "Tracker request"
            );

            let _ = init_server(&client, server_args).await;
        }
    }
}
