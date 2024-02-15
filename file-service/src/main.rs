use anyhow::Error;
use axum::{routing::get, Router};
use clap::Parser;
use file_service::file_server::{
    cost::cost, initialize_server_context, status::status, util::graphql_playground,
};
use indexer_common::indexer_service::http::{
    IndexerService, IndexerServiceOptions, IndexerServiceRelease,
};

use tracing::error;

mod cli;
mod config;
pub mod database;
pub mod file_server;

/// Run the subgraph indexer service
#[tokio::main]
async fn main() -> Result<(), Error> {
    // console_subscriber::init();

    file_exchange::config::init_tracing("pretty").expect("Initialize logger");

    // Parse command line and environment arguments
    let cli = config::Cli::parse();
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
            .route("/files-cost", get(graphql_playground).post(cost))
            .route("/files-status", get(graphql_playground).post(status))
            // .route("/admin", post(admin::handle_admin_request))
            .with_state(state),
    })
    .await
}
