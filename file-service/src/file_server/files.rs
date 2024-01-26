// #![cfg(feature = "acceptor")]
use http::header::CONTENT_RANGE;

use file_exchange::errors::{Error, ServerError};

use crate::file_server::util::{Health, Operator};
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::{Body, Request, Response, StatusCode};

use super::{
    range::{parse_range_header, serve_file, serve_file_range},
    ServerContext,
};

// Serve file requests
pub async fn file_service(
    id: DeploymentId,
    req: &Request<Body>,
    context: &ServerContext,
) -> Result<Response<Body>, Error> {
    tracing::debug!("Received file range request");
    let context_ref = context.lock().await;
    tracing::debug!(
        bundles = tracing::field::debug(&context_ref),
        id,
        "Received file range request"
    );

    // Validate the auth token
    let auth_token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|t| t.to_str().ok());

    let free = context_ref.free_query_auth_token.is_none()
        || (auth_token.is_some()
            && context_ref.free_query_auth_token.is_some()
            && auth_token.unwrap() == context_ref.free_query_auth_token.as_deref().unwrap());

    if !free {
        tracing::warn!("Respond with unauthorized query");
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Paid service is not implemented, need free query authentication".into())
            .unwrap());
    }

    let requested_bundle = match context_ref.bundles.get(id) {
        Some(s) => s.clone(),
        None => {
            tracing::debug!(
                server_context = tracing::field::debug(&context_ref),
                id,
                "Requested bundle is not served locally"
            );
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Bundle not found".into())
                .unwrap());
        }
    };

    match req.headers().get("file_hash") {
        Some(hash) if hash.to_str().is_ok() => {
            let mut file_path = requested_bundle.local_path.clone();
            let file_manifest = match requested_bundle
                .file_manifests
                .iter()
                .find(|file| file.meta_info.hash == hash.to_str().unwrap())
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
            match req.headers().get(CONTENT_RANGE) {
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
