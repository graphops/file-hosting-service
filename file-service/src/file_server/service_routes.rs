// #![cfg(feature = "acceptor")]

use indexer_common::indexer_service::http::IndexerServiceImpl;
use thegraph::types::DeploymentId;

use crate::file_server::util::{Health, Operator};
use file_exchange::errors::{Error, ServerError};
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::{Body, Response, StatusCode};

use super::{
    range::{parse_range_header, serve_file, serve_file_range},
    ServerContext,
};

/// Endpoint for server health
pub async fn health() -> Result<Response<Body>, Error> {
    let health = Health { healthy: true };
    let health_json = serde_json::to_string(&health).map_err(Error::JsonError)?;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(health_json))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

/// Endpoint for package version
pub async fn version(context: &ServerContext) -> Result<Response<Body>, Error> {
    let version = context.state.lock().await.release.version.clone();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(version))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

/// Endpoint for cost to download per byte
pub async fn cost(context: &ServerContext) -> Result<Response<Body>, Error> {
    let price = context.state.lock().await.price_per_byte.to_string();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(price))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

/// Endpoint for status availability
pub async fn status(context: &ServerContext) -> Result<Response<Body>, Error> {
    let bundle_mapping = context.state.lock().await.bundles.clone();
    let bundle_ipfses: Vec<String> = bundle_mapping
        .keys()
        .map(|i| i.to_owned())
        .collect::<Vec<String>>();
    let json = serde_json::to_string(&bundle_ipfses).map_err(Error::JsonError)?;

    tracing::debug!(json, "Serving status");
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

// Define a handler function for the `/info` route
pub async fn operator_info(context: &ServerContext) -> Result<Response<Body>, Error> {
    let public_key = context.state.lock().await.operator_public_key.clone();
    let operator = Operator { public_key };
    let json = serde_json::to_string(&operator).map_err(Error::JsonError)?;
    tracing::debug!(json, "Operator info response");
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

// Serve file requests
pub async fn file_service(
    id: DeploymentId,
    req: &<ServerContext as IndexerServiceImpl>::Request,
    context: &ServerContext,
) -> Result<Response<Body>, Error> {
    tracing::debug!("Received file range request");
    let context_ref = context.state.lock().await;
    tracing::debug!(
        bundles = tracing::field::debug(&context_ref),
        id = tracing::field::debug(&id.to_string()),
        "Received file range request"
    );

    let requested_bundle = match context_ref.bundles.get(&id.to_string()) {
        Some(s) => s.clone(),
        None => {
            tracing::debug!(
                server_context = tracing::field::debug(&context_ref),
                id = tracing::field::debug(&id.to_string()),
                "Requested bundle is not served locally"
            );
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Bundle not found".into())
                .unwrap());
        }
    };

    match req.get("file-hash") {
        Some(hash) if hash.as_str().is_some() => {
            let mut file_path = requested_bundle.local_path.clone();
            let file_manifest = match requested_bundle
                .file_manifests
                .iter()
                .find(|file| file.meta_info.hash == hash.as_str().unwrap())
            {
                Some(c) => c,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body("File manifest not found".into())
                        .unwrap())
                }
            };
            file_path.push(file_manifest.meta_info.name.clone());
            // Parse the range header to get the start and end bytes
            match req.get("content-range") {
                Some(r) => {
                    tracing::debug!("Parse content range header");
                    let range = parse_range_header(r)?;
                    //TODO: validate receipt
                    serve_file_range(&file_path, range).await
                }
                None => {
                    tracing::info!("Serve file");
                    serve_file(&file_path).await
                }
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .body("Missing required file_manifest_hash header".into())
            .unwrap()),
    }
}
