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
    tracing::debug!("Received file range request");
    let context_ref = context.state.lock().await;
    tracing::debug!(
        id = tracing::field::debug(&id.to_string()),
        "Received file range request"
    );

    let requested_bundle = match context_ref.bundles.get(&id.to_string()) {
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
                    let range = parse_range_header(r)?;
                    //TODO: validate receipt
                    serve_file_range(&file_path, range).await
                }
                None => serve_file(&file_path).await,
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .body("Missing required file_manifest_hash header".into())
            .unwrap()),
    }
}
