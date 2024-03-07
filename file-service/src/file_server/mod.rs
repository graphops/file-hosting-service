// #![cfg(feature = "acceptor")]
use axum::{
    async_trait,
    response::{IntoResponse, Response},
};

use sqlx::PgPool;
use std::sync::Arc;
use std::{collections::HashMap, string::FromUtf8Error};
use thegraph::types::{Attestation, DeploymentId};
// use sqlx::PgPool;
use indexer_common::indexer_service::http::{IndexerServiceImpl, IndexerServiceResponse};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{config::Config, database};
use file_exchange::manifest::{
    ipfs::IpfsClient, manifest_fetcher::read_bundle, validate_bundle_entries, LocalBundle,
};
use file_exchange::util::public_key;
use file_exchange::{errors::Error, manifest::local_file_system::Store};
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::StatusCode;

pub mod cost;
pub mod range;
pub mod service;
pub mod status;
pub mod util;

#[derive(Clone)]
pub struct ServerState {
    pub client: IpfsClient,
    pub operator_public_key: String,
    pub bundles: Arc<Mutex<HashMap<String, LocalBundle>>>, // Keyed by IPFS hash, valued by Bundle and Local path
    pub admin_auth_token: Option<String>,                  // Add bearer prefix
    pub config: Config,
    pub database: PgPool,
    pub cost_schema: crate::file_server::cost::CostSchema,
    pub status_schema: crate::file_server::status::StatusSchema,
    pub store: Store,
}

#[derive(Clone)]
pub struct ServerContext {
    pub state: Arc<ServerState>,
}

impl ServerContext {
    pub fn new(state: Arc<ServerState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl IndexerServiceImpl for ServerContext {
    type Error = FileServiceError;
    type Request = serde_json::Value;
    type Response = FileServiceResponse;
    type State = ServerState;

    async fn process_request(
        &self,
        deployment: DeploymentId,
        request: Self::Request,
    ) -> Result<(Self::Request, Self::Response), Self::Error> {
        //TODO: consider routing through file level IPFS
        // path if path.starts_with("/bundles/id/") => {
        // }
        tracing::info!("do file service");
        let body = service::file_service(deployment, &request, self)
            .await
            .map_err(FileServiceError::QueryForwardingError)?;
        let response = FileServiceResponse { inner: body };
        Ok((request, response))
    }
}

/// Function to initialize the file hosting server
pub async fn initialize_server_context(config: Config) -> Result<ServerContext, Error> {
    tracing::debug!(
        config = tracing::field::debug(&config),
        "Initializing server context"
    );

    let client = if let Ok(client) = IpfsClient::new(&config.server.ipfs_gateway) {
        client
    } else {
        IpfsClient::localhost()
    };
    let bundle_entries = validate_bundle_entries(config.server.bundles.clone())?;
    tracing::debug!(
        entries = tracing::field::debug(&bundle_entries),
        "Validated bundle entries"
    );

    let admin_auth_token = config
        .server
        .admin_auth_token
        .clone()
        .map(|token| format!("Bearer {}", token));

    build_info::build_info!(fn build_info);
    // Add the file to the service availability endpoint
    // This would be part of your server state initialization
    let server_state = ServerState {
        config: config.clone(),
        client: client.clone(),
        bundles: Arc::new(Mutex::new(HashMap::new())),
        admin_auth_token,
        operator_public_key: public_key(&config.common.indexer.operator_mnemonic)
            .expect("Failed to initiate with operator wallet"),
        database: database::connect(&config.common.database.postgres_url).await,
        cost_schema: cost::build_schema().await,
        status_schema: status::build_schema().await,
        store: Store::new(&config.server.main_directory).expect("Storage system"),
    };

    // Fetch the file using IPFS client
    for (ipfs_hash, local_path) in bundle_entries {
        let bundle = read_bundle(&server_state.client, &ipfs_hash).await?;
        // let bundle = read_bundle(&server_state.client, &ipfs_hash, local_path).await?;
        // let _ = bundle.validate_local_bundle();

        server_state
            .bundles
            .lock()
            .await
            .insert(bundle.ipfs_hash.clone(), LocalBundle { bundle, local_path });
    }

    // Return the server state wrapped in an Arc for thread safety
    Ok(ServerContext::new(Arc::new(server_state)))
}

#[derive(Debug, Error)]
pub enum FileServiceError {
    #[error("Invalid status query: {0}")]
    InvalidStatusQuery(anyhow::Error),
    #[error("Internal server error: {0}")]
    StatusQueryError(anyhow::Error),
    #[error("Invalid deployment: {0}")]
    InvalidDeployment(String),
    #[error("Failed to process query: {0}")]
    QueryForwardingError(Error),
    #[error("Failed to provide query string: {0}")]
    QueryParseError(FromUtf8Error),
    #[error("Admin request failed: {0}")]
    AdminError(Error),
}

impl From<&FileServiceError> for StatusCode {
    fn from(err: &FileServiceError) -> Self {
        use FileServiceError::*;
        match err {
            InvalidStatusQuery(_) => StatusCode::BAD_REQUEST,
            StatusQueryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvalidDeployment(_) => StatusCode::BAD_REQUEST,
            QueryForwardingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            QueryParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AdminError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

// Convert `FileServiceError` into a response.
impl IntoResponse for FileServiceError {
    fn into_response(self) -> Response {
        (StatusCode::from(&self), self.to_string()).into_response()
    }
}

// #[derive(Debug)]
pub struct FileServiceResponse {
    inner: hyper::Response<hyper::Body>,
}

impl IndexerServiceResponse for FileServiceResponse {
    type Data = hyper::Response<hyper::Body>;
    type Error = FileServiceError; // not used

    fn is_attestable(&self) -> bool {
        // self.attestable
        false
    }

    fn as_str(&self) -> Result<&str, Self::Error> {
        // String::from_utf8(self.inner.into_body().data().).map(|s| s.as_str()).map_err(|e|
        //     FileServiceError::QueryParseError(e))
        Ok("Not represented as str")
    }

    fn finalize(self, _attestation: Option<Attestation>) -> Self::Data {
        self.inner
    }
}
