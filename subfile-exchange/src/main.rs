use dotenv::dotenv;

use subfile_exchange::{
    config::{Cli, OnchainAction, Role},
    publisher::SubfilePublisher,
    subfile::ipfs::IpfsClient,
    subfile_client::SubfileDownloader,
    transaction_manager::TransactionManager,
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
            let downloader = SubfileDownloader::new(client, config).await;

            // Send range request
            match downloader.download_subfile().await {
                Ok(res) => {
                    tracing::info!("Download result: {:#?}", res);
                }
                Err(e) => {
                    tracing::error!(err = e.to_string());
                }
            }
        }
        Role::Publisher(config) => {
            tracing::info!(config = tracing::field::debug(&config), "Publisher request");

            let publisher = SubfilePublisher::new(client, config);

            match publisher.publish().await {
                Ok(r) => {
                    tracing::info!(result = tracing::field::debug(&r), "Published");
                }
                Err(e) => {
                    tracing::error!(error = tracing::field::debug(&e), "Failed to build");
                }
            }
        }
        Role::Wallet(wallet_args) => {
            tracing::info!(
                server = tracing::field::debug(&wallet_args),
                "Use the provided wallet to send transactions"
            );

            let transaction_manager = TransactionManager::new(wallet_args)
                .await
                .expect("Cannot initate transaction manager");

            match transaction_manager.args.action.clone() {
                Some(OnchainAction::Allocate(allocate_args)) => {
                    let (allocation_id, tx_receipt) = transaction_manager
                        .allocate(
                            &allocate_args.deployment_ipfs,
                            allocate_args.tokens,
                            allocate_args.epoch,
                        )
                        .await
                        .unwrap();
                    tracing::info!(
                        allocation_id = tracing::field::debug(&allocation_id),
                        tx_receipt = tracing::field::debug(&tx_receipt),
                        "Allocation transaction finished"
                    );
                }
                Some(OnchainAction::Unallocate(_unallocate_args)) => {
                    todo!()
                }
                None => {}
            }
        }
    }
}
