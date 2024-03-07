// #![cfg(feature = "acceptor")]

use indexer_common::indexer_service::http::IndexerServiceImpl;
use thegraph::types::DeploymentId;

use file_exchange::errors::Error;
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::{Body, Response, StatusCode};

use super::{
    range::{parse_range_header, serve_file, serve_file_range},
    ServerContext,
};

// Serve file requests
pub async fn file_service(
    id: DeploymentId,
    req: &<ServerContext as IndexerServiceImpl>::Request,
    context: &ServerContext,
) -> Result<Response<Body>, Error> {
    tracing::debug!(
        id = tracing::field::debug(&id.to_string()),
        "Received file range request"
    );

    let local_bundle = match context.state.bundles.lock().await.get(&id.to_string()) {
        Some(s) => s.clone(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Bundle not found".into())
                .unwrap());
        }
    };

    match req.get("file-hash") {
        Some(hash) if hash.as_str().is_some() => {
            let file_manifest = match local_bundle
                .bundle
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
            // Parse the range header to get the start and end bytes
            match req.get("content-range") {
                Some(r) => {
                    let range = parse_range_header(r)?;
                    //TODO: validate receipt
                    serve_file_range(
                        context.state.store.clone(),
                        &file_manifest.meta_info.name,
                        &local_bundle.local_path,
                        range,
                    )
                    .await
                }
                None => {
                    serve_file(
                        context.state.store.clone(),
                        &file_manifest.meta_info.name,
                        &local_bundle.local_path,
                    )
                    .await
                }
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .body("Missing required file_manifest_hash header".into())
            .unwrap()),
    }
}
