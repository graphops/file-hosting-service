use hyper::body::to_bytes;
use hyper::{Body, Request, Response, StatusCode};
use serde_json::{json, Value};

use crate::config::{is_valid_ipfs_hash, validate_subfile_entries};
use crate::errors::Error;
use crate::subfile_reader::read_subfile;

use super::{create_error_response, ServerContext};

/// Handle admin requests
pub async fn handle_admin_request(
    req: Request<hyper::Body>,
    context: &ServerContext,
) -> Result<hyper::Response<hyper::Body>, Error> {
    // Validate the auth token
    tracing::debug!("Received admin request");
    let server_auth_token = context.lock().await.admin_auth_token.clone();
    let auth_token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|t| t.to_str().ok());

    let authorized = server_auth_token.is_none()
        || (auth_token.is_some()
            && server_auth_token.is_some()
            && auth_token.unwrap() == server_auth_token.as_deref().unwrap());

    if !authorized {
        tracing::warn!("Respond unauthorized");
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Require admin authentication".into())
            .map_err(|e| {
                Error::ServerError(crate::errors::ServerError::BuildResponseError(
                    e.to_string(),
                ))
            });
    }

    let body_bytes = to_bytes(req.into_body()).await.map_err(|e| {
        Error::ServerError(crate::errors::ServerError::RequestBodyError(e.to_string()))
    })?;

    let json: Value = serde_json::from_slice(&body_bytes).map_err(Error::JsonError)?;

    let method = json.get("method").and_then(Value::as_str).ok_or_else(|| {
        Error::ServerError(crate::errors::ServerError::MethodParseError(
            "Method not found in request".to_string(),
        ))
    })?;
    let params = json.get("params");

    tracing::info!("Received valid/authorized subfiles management request");

    match method {
        "get_subfiles" => get_subfiles(context).await,
        "add_subfile" => {
            add_subfile(
                params
                    .ok_or_else(|| {
                        Error::ServerError(crate::errors::ServerError::ParamsParseError(
                            "Params not found in request".to_string(),
                        ))
                    })?
                    .clone(),
                context,
            )
            .await
        }
        "remove_subfile" => {
            remove_subfile(
                params
                    .ok_or_else(|| {
                        Error::ServerError(crate::errors::ServerError::ParamsParseError(
                            "Params not found in request".to_string(),
                        ))
                    })?
                    .clone(),
                context,
            )
            .await
        }
        _ => Ok(hyper::Response::builder()
            .status(hyper::StatusCode::METHOD_NOT_ALLOWED)
            .body("Method not supported".into())
            .map_err(|e| {
                Error::ServerError(crate::errors::ServerError::BuildResponseError(
                    e.to_string(),
                ))
            })?),
    }
}

//TODO: rich the details
/// Function to retrieve all subfiles and their details
async fn get_subfiles(context: &ServerContext) -> Result<Response<Body>, Error> {
    let server_state = context.lock().await;
    // Create a JSON object or array containing the subfiles' details
    let subfiles_info = server_state
        .subfiles
        .iter()
        .map(|(ipfs_hash, subfile)| json!({ "ipfs_hash": ipfs_hash, "subfile": subfile }))
        .collect::<Vec<_>>();
    drop(server_state);

    let body = serde_json::to_string(&subfiles_info).map_err(Error::JsonError)?;
    tracing::trace!("Built get_subfile response");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(body.into())
        .unwrap())
}

/// Add a subfile to the server state
async fn add_subfile(params: Value, context: &ServerContext) -> Result<Response<Body>, Error> {
    let entries: Vec<String> = serde_json::from_value(params).map_err(Error::JsonError)?;

    // Validate before adding to the server state
    let subfile_entries = match validate_subfile_entries(entries) {
        Ok(s) => s,
        Err(e) => {
            return Ok(create_error_response(
                &e.to_string(),
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let mut server_state = context.lock().await;
    for (ipfs_hash, local_path) in subfile_entries {
        let subfile = read_subfile(&server_state.client, &ipfs_hash, local_path).await?;
        if let Err(e) = subfile.validate_local_subfile() {
            return Ok(create_error_response(
                &e.to_string(),
                StatusCode::BAD_REQUEST,
            ));
        };

        server_state
            .subfiles
            .insert(subfile.ipfs_hash.clone(), subfile);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Subfile(s) added successfully".into())
        .unwrap())
}

/// Remove a subfile from the server state
async fn remove_subfile(params: Value, context: &ServerContext) -> Result<Response<Body>, Error> {
    let ipfs_hashes: Vec<String> = serde_json::from_value(params).map_err(Error::JsonError)?;

    for ipfs_hash in &ipfs_hashes {
        match !is_valid_ipfs_hash(ipfs_hash) {
            true => {
                return Ok(create_error_response(
                    &format!("Invalid IPFS hash: {}", ipfs_hash),
                    StatusCode::BAD_REQUEST,
                ))
            }
            false => (),
        }
    }

    // Access the server state
    let mut server_state = context.lock().await;

    // Remove the valid IPFS hashes from the server state's subfiles
    for ipfs_hash in ipfs_hashes {
        server_state.subfiles.remove(&ipfs_hash);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Subfile(s) removed successfully".into())
        .unwrap())
}
