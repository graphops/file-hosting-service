use dotenv::dotenv;

use subfile_service::{config::Config, subfile_server::init_server};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config: Config = Config::args();

    let _ = init_server(config).await;
}
