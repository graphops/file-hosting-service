use anyhow::Error;
use axum::{routing::post, Router};
use clap::Parser;
use file_service::file_server::{cost::cost, initialize_server_context, status::status};
use indexer_common::indexer_service::http::{
    IndexerService, IndexerServiceOptions, IndexerServiceRelease,
};

use tracing::error;

mod cli;
mod config;
pub mod database;
pub mod file_server;

use cli::Cli;

/// Run the subgraph indexer service
#[tokio::main]
async fn main() -> Result<(), Error> {
    file_exchange::config::init_tracing("pretty").expect("Initialize logger");

    // Parse command line and environment arguments
    let cli = Cli::parse();
    let config = match file_service::config::Config::load(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "Invalid configuration file `{}`: {}",
                cli.config.display(),
                e
            );
            std::process::exit(1);
        }
    };

    // Parse basic configurations
    build_info::build_info!(fn build_info);
    let release = IndexerServiceRelease::from(build_info());

    let state = initialize_server_context(config.clone())
        .await
        .expect("Failed to initiate bundle server");

    IndexerService::run(IndexerServiceOptions {
        release,
        config: config.clone().common,
        url_namespace: "files",
        metrics_prefix: "files",
        service_impl: state.clone(),
        extra_routes: Router::new()
            .route("/files-cost", post(cost))
            .route("/files-status", post(status))
            // .route("/admin", post(admin::handle_admin_request))
            // "/admin" => handle_admin_request(req, &context).await,
            .with_state(state),
    })
    .await
}
