// #![cfg(feature = "acceptor")]
use axum::{
    async_trait,
    response::{IntoResponse, Response},
};

use std::sync::Arc;
use std::{collections::HashMap, string::FromUtf8Error};
use thegraph::types::{Attestation, DeploymentId};
// use sqlx::PgPool;
use indexer_common::indexer_service::http::{IndexerServiceImpl, IndexerServiceResponse};
use thiserror::Error;
use tokio::sync::Mutex;

use file_exchange::errors::{Error, ServerError};
use file_exchange::manifest::{
    ipfs::IpfsClient, manifest_fetcher::read_bundle, validate_bundle_entries, Bundle,
};

use crate::config::Config;
// use crate::config::{Config, ServerArgs};
use crate::file_server::util::public_key;
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::StatusCode;

pub mod admin;
pub mod cost;
pub mod range;
pub mod service_routes;
pub mod status;
pub mod util;

#[derive(Debug, Clone)]
pub struct ServerState {
    pub client: IpfsClient,
    pub operator_public_key: String,
    pub bundles: HashMap<String, Bundle>, // Keyed by IPFS hash
    pub release: util::PackageVersion,
    pub free_query_auth_token: Option<String>, // Add bearer prefix
    pub admin_auth_token: Option<String>,      // Add bearer prefix
    pub price_per_byte: f32,

    pub config: Config,
    // pub database: PgPool,
    // pub cost_schema: crate::file_server::cost::CostSchema,
}

#[derive(Debug, Clone)]
pub struct ServerContext {
    state: Arc<Mutex<ServerState>>,
}

impl ServerContext {
    pub fn new(state: Arc<Mutex<ServerState>>) -> Self {
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
        tracing::trace!("do file service");
        let body = service_routes::file_service(deployment, &request, self)
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

    let free_query_auth_token = config
        .common
        .server
        .free_query_auth_token
        .clone()
        .map(|token| format!("Bearer {}", token));
    let admin_auth_token = config
        .server
        .admin_auth_token
        .clone()
        .map(|token| format!("Bearer {}", token));

    build_info::build_info!(fn build_info);
    // Add the file to the service availability endpoint
    // This would be part of your server state initialization
    let mut server_state = ServerState {
        config: config.clone(),
        client: client.clone(),
        bundles: HashMap::new(),
        release: util::PackageVersion::from(build_info()),
        free_query_auth_token,
        admin_auth_token,
        operator_public_key: public_key(&config.common.indexer.operator_mnemonic)
            .expect("Failed to initiate with operator wallet"),
        price_per_byte: config.server.price_per_byte,
    };

    // Fetch the file using IPFS client
    for (ipfs_hash, local_path) in bundle_entries {
        let bundle = read_bundle(&server_state.client, &ipfs_hash, local_path).await?;
        let _ = bundle.validate_local_bundle();

        server_state
            .bundles
            .insert(bundle.ipfs_hash.clone(), bundle);
    }

    // Return the server state wrapped in an Arc for thread safety
    Ok(ServerContext::new(Arc::new(Mutex::new(server_state))))
}

/// Create an admin error response
pub fn admin_error_response(msg: &str) -> FileServiceError {
    FileServiceError::AdminError(Error::ServerError(ServerError::InvalidAuthentication(
        msg.to_string(),
    )))
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
