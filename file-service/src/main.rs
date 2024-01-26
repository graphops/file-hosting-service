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

// #[derive(Debug, Error)]
// pub enum FileServiceError {
//     #[error("Invalid status query: {0}")]
//     InvalidStatusQuery(Error),
//     #[error("Unsupported status query fields: {0:?}")]
//     UnsupportedStatusQueryFields(Vec<String>),
//     #[error("Internal server error: {0}")]
//     StatusQueryError(Error),
//     #[error("Invalid deployment: {0}")]
//     InvalidDeployment(DeploymentId),
//     #[error("Failed to process query: {0}")]
//     QueryForwardingError(reqwest::Error),
// }

// impl From<&FileServiceError> for StatusCode {
//     fn from(err: &FileServiceError) -> Self {
//         use FileServiceError::*;
//         match err {
//             InvalidStatusQuery(_) => StatusCode::BAD_REQUEST,
//             UnsupportedStatusQueryFields(_) => StatusCode::BAD_REQUEST,
//             StatusQueryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
//             InvalidDeployment(_) => StatusCode::BAD_REQUEST,
//             QueryForwardingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
//         }
//     }
// }

// // Convert `FileServiceError` into a response.
// impl IntoResponse for FileServiceError {
//     fn into_response(self) -> Response {
//         (StatusCode::from(&self), self.to_string()).into_response()
//     }
// }

// #[derive(Debug)]
// struct FileServiceResponse {
//     inner: String,
//     attestable: bool,
// }

// impl FileServiceResponse {
//     fn new(inner: String, attestable: bool) -> Self {
//         Self { inner, attestable }
//     }
// }

// impl IndexerServiceResponse for FileServiceResponse {
//     type Data = Json<Value>;
//     type Error = FileServiceError; // not used

//     fn is_attestable(&self) -> bool {
//         self.attestable
//     }

//     fn as_str(&self) -> Result<&str, Self::Error> {
//         Ok(self.inner.as_str())
//     }

//     fn finalize(self, attestation: Option<Attestation>) -> Self::Data {
//         Json(json!({
//             "graphQLResponse": self.inner,
//             "attestation": attestation
//         }))
//     }
// }

// pub struct FileServiceState {
//     pub config: Config,
//     pub database: PgPool,
//     pub cost_schema: routes::cost::CostSchema,

// }

// struct ServerContext {
//     state: Arc<FileServiceState>,
// }

// impl ServerContext {
//     fn new(state: Arc<FileServiceState>) -> Self {
//         Self { state }
//     }
// }

// #[async_trait]
// impl IndexerServiceImpl for ServerContext {
//     type Error = FileServiceError;
//     type Request = serde_json::Value;
//     type Response = hyper::Response;
//     type State = FileServiceState;

//     async fn process_request(
//         &self,
//         deployment: DeploymentId,
//         request: Self::Request,
//     ) -> Result<(Self::Request, Self::Response), Self::Error> {
//         let deployment_url = Url::parse(&format!(
//             "{}/files/id/{}",
//             &self.state.graph_node_query_base_url, deployment
//         ))
//         .map_err(|_| FileServiceError::InvalidDeployment(deployment))?;

//         let response = self
//             .state
//             .graph_node_client
//             .post(deployment_url)
//             .json(&request)
//             .send()
//             .await
//             .map_err(FileServiceError::QueryForwardingError)?;

//         let attestable = response
//             .headers()
//             .get("graph-attestable")
//             .map_or(false, |value| {
//                 value.to_str().map(|value| value == "true").unwrap_or(false)
//             });

//         let body = response
//             .text()
//             .await
//             .map_err(FileServiceError::QueryForwardingError)?;

//         Ok((request, FileServiceResponse::new(body, attestable)))
//     }
// }

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
