use dotenv::dotenv;

use subfile_exchange::ipfs::IpfsClient;
use subfile_service::{config::Cli, subfile_server::init_server};

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

    let _ = init_server(&client, cli.server).await;
}
