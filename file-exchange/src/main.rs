use dotenv::dotenv;

use file_exchange::{
    config::{Cli, OnchainAction, Role},
    download_client::Downloader,
    graphql::network_query::current_epoch,
    manifest::ipfs::IpfsClient,
    publisher::ManifestPublisher,
    transaction_manager::TransactionManager,
    util::store_map_as_json,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli: Cli = Cli::args();
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
            let progress_file = config.progress_file.clone();
            // Create client
            let downloader = Downloader::new(client, config).await;

            if let Some(cache) = progress_file {
                let chunks = downloader.target_chunks.clone();
                ctrlc::set_handler(move || {
                    tracing::info!("CTRL+C pressed. Store progress cache to json");

                    let map = chunks.lock().unwrap();
                    match store_map_as_json(&map, &cache) {
                        Ok(_) => println!("Data successfully saved"),
                        Err(e) => eprintln!("Failed to save progress: {}", e),
                    }

                    std::process::exit(0); // Exit the process
                })
                .expect("Error setting Ctrl-C handler");
            }

            // Send range request
            match downloader.download_bundle().await {
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

            let publisher = ManifestPublisher::new(client, config);

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

            let result = match transaction_manager.args.action.clone() {
                Some(OnchainAction::Allocate(allocate_args)) => {
                    let epoch = current_epoch(
                        &reqwest::Client::new(),
                        &transaction_manager.args.network_subgraph,
                        1,
                    )
                    .await
                    .expect("Fetch epoch number");
                    transaction_manager
                        .allocate(&allocate_args.deployment_ipfs, allocate_args.tokens, epoch)
                        .await
                }
                Some(OnchainAction::Unallocate(unallocate_args)) => {
                    transaction_manager
                        .unallocate(&unallocate_args.allocation_id)
                        .await
                }
                Some(OnchainAction::Deposit(deposit_args)) => {
                    transaction_manager
                        .deposit(&deposit_args.receiver, &deposit_args.tokens)
                        .await
                }
                Some(OnchainAction::DepositMany(deposit_many_args)) => {
                    transaction_manager
                        .deposit_many(deposit_many_args.receivers, deposit_many_args.tokens)
                        .await
                }
                Some(OnchainAction::Withdraw(withdraw_args)) => {
                    transaction_manager.withdraw(&withdraw_args.receiver).await
                }
                Some(OnchainAction::Approve(approve_args)) => {
                    transaction_manager
                        .approve_escrow(&approve_args.tokens)
                        .await
                }
                None => {
                    panic!("No onchain command provided (later add general status return)")
                }
            };
            tracing::info!(
                result = tracing::field::debug(&result),
                "Transaction result"
            );
        }
    }
}
