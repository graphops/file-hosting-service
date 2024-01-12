use dotenv::dotenv;

use file_service::{config::Config, file_server::init_server};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config: Config = Config::args();

    let _ = init_server(config).await;
}
