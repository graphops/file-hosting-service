// use file_exchange::errors::ServerError;
// use crate::file_server::{admin_error_response, FileServiceError};
// use file_exchange::{
//     errors::Error,
//     manifest::{ipfs::is_valid_ipfs_hash, manifest_fetcher::read_bundle, validate_bundle_entries},
// };
// use hyper::body::to_bytes;
// use hyper::{Body, Request, Response, StatusCode};
// use serde_json::{json, Value};
// use indexer_common::indexer_service::http::IndexerServiceImpl;
// use axum::{response::IntoResponse, extract::State, Json, body::Bytes};

// use super::ServerContext;

// /// Handle admin requests
// #[axum_macros::debug_handler]
// pub async fn handle_admin_request(
//     // State(context): State<ServerContext>,
//     req: Bytes,
// )
// {
// // Validate the auth token
//     tracing::debug!("Received admin request");
//     let server_auth_token = context.state.lock().await.admin_auth_token.clone();
//     let auth_token = req
//         .get("authorization")
//         .and_then(|t| t.as_str());

//     let authorized = server_auth_token.is_none()
//         || (auth_token.is_some()
//             && server_auth_token.is_some()
//             && auth_token.unwrap() == server_auth_token.as_deref().unwrap());

//     if !authorized {
//         tracing::warn!("Respond unauthorized");
//         let response_body = json!({ "error": "Require admin authentication" });
//         return Err(FileServiceError::AdminError(Error::ServerError(ServerError::InvalidAuthentication("Unauthorized as an admin".to_string()))))
//     }

//     // let method = req.get("method").and_then(Value::as_str).ok_or_else(|| {
//     //     Error::ServerError(ServerError::MethodParseError(
//     //         "Method not found in request".to_string(),
//     //     ))
//     // })?;
//     // let params = req.get("params");

//     // Ok((method.to_string(), params.cloned()))
//     // let (method, params) = match parse_admin_request(req).await {
//     //     Ok(r) => r,
//     //     Err(e) => {
//     //         return Ok(admin_error_response(
//     //             &e.to_string(),
//     //         ))
//     //     }
//     // };

//     // tracing::debug!(
//     //     method = tracing::field::debug(&method),
//     //     params = tracing::field::debug(&params),
//     //     "Received valid/authorized bundles management request"
//     // );

//     match req.get("method") {
//         // Some(hash) if hash.as_str().is_some() && hash.as_str().unwrap() == "get_bundles"
//             // => get_bundles(context).await,
//         Some(hash) if hash.as_str().is_some() && hash.as_str().unwrap() == "add_bundle"
//             => {
//             add_bundle(req, context).await},
//         // Some(hash) if hash.as_str().is_some() && hash.as_str().unwrap() == "remove_bundle"
//         //     => {remove_bundle(req, context).await},
//         // Some(hash) if hash.as_str().is_some() && hash.as_str().unwrap() == "update_price_per_byte"
//         //     => update_price_per_byte(req, context).await,
//         _ => Err(FileServiceError::AdminError(Error::ServerError(ServerError::MethodParseError("Unsupported admin method".to_string())))),
//     }
// }

// async fn parse_admin_request(req: &serde_json::Value) -> Result<(String, Option<Value>), Error> {
//     // let body_bytes = to_bytes(req.into_body())
//     //     .await
//     //     .map_err(|e| Error::ServerError(ServerError::RequestBodyError(e.to_string())))?;

//     // let json: Value = serde_json::from_slice(&body_bytes).map_err(Error::JsonError)?;

//     let method = req.get("method").and_then(Value::as_str).ok_or_else(|| {
//         Error::ServerError(ServerError::MethodParseError(
//             "Method not found in request".to_string(),
//         ))
//     })?;
//     let params = req.get("params");

//     Ok((method.to_string(), params.cloned()))
//     // Create a JSON object or array containing the bundles' details
//     let bundles_info = server_state
//         .bundles
//         .iter()
//         .map(|(ipfs_hash, bundle)| json!({ "ipfs_hash": ipfs_hash, "bundle": bundle }))
//         .collect::<Vec<_>>();
//     drop(server_state);

//     let body = match serde_json::to_string(&bundles_info).map_err(Error::JsonError) {
//         Ok(b) => b,
//         Err(e) => {
//             return Err(admin_error_response(
//                 &e.to_string(),
//             ))
//         }
//     };
//     tracing::trace!("Built get_bundle response");

//     Ok(Response::builder()
//         .status(StatusCode::OK)
//         .body(body.into())
//         .unwrap())
// }

// /// Add a bundle to the server state
// async fn add_bundle(
//     req: &Value,
//     context: &ServerContext,
// ) -> Result<impl IntoResponse, FileServiceError> {
//     let entries: Vec<String> = match req["params"].as_str() {
//         Some(p) => serde_json::from_str::<Vec<String>>(p).map_err(|e| FileServiceError::AdminError(Error::ServerError(ServerError::ParamsParseError(e.to_string())))),
//         None => Err(FileServiceError::AdminError(Error::ServerError(ServerError::ParamsParseError("The 'params' field is not provided".to_string()))))
//     }?;

//     // Validate before adding to the server state
//     let bundle_entries = match validate_bundle_entries(entries) {
//         Ok(s) => s,
//         Err(e) => {
//             return Err(admin_error_response(
//                 &e.to_string(),
//             ))
//         }
//     };
//     let mut server_state = context.state.lock().await;
//     for (ipfs_hash, local_path) in bundle_entries {
//         let bundle = match read_bundle(&server_state.client, &ipfs_hash, local_path).await {
//             Ok(s) => s,
//             Err(e) => {
//                 return Err(admin_error_response(
//                     &e.to_string(),
//                 ))
//             }
//         };
//         if let Err(e) = bundle.validate_local_bundle() {
//             return Err(admin_error_response(
//                 &e.to_string(),
//             ));
//         };

//         server_state
//             .bundles
//             .insert(bundle.ipfs_hash.clone(), bundle);
//     };
//     Ok(Json(json!("Bundle(s) added successfully")))
// }

// /// Remove a bundle from the server state
// async fn remove_bundle(
//     params: Option<&Value>,
//     context: &ServerContext,
// ) -> Result<Response<Body>, FileServiceError> {
//     let params = match params {
//         Some(p) => p,
//         None => {
//             return Ok(admin_error_response(
//                 "Missing params",
//             ))
//         }
//     };
//     let ipfs_hashes: Vec<String> = match serde_json::from_value(params.clone()).map_err(Error::JsonError) {
//         Ok(h) => h,
//         Err(e) => {
//             return Ok(admin_error_response(
//                 &e.to_string(),
//             ))
//         }
//     };

//     for ipfs_hash in &ipfs_hashes {
//         match !is_valid_ipfs_hash(ipfs_hash) {
//             true => {
//                 return Ok(admin_error_response(
//                     &format!("Invalid IPFS hash: {}", ipfs_hash),
//                 ))
//             }
//             false => (),
//         }
//     }

//     // Access the server state
//     let mut server_state = context.state.lock().await;

//     // Remove the valid IPFS hashes from the server state's bundles
//     for ipfs_hash in ipfs_hashes {
//         server_state.bundles.remove(&ipfs_hash);
//     }

//     Ok(Response::builder()
//         .status(StatusCode::OK)
//         .body("Bundle(s) removed successfully".into())
//         .unwrap())
// }

// /// Update price per byte
// async fn update_price_per_byte(
//     params: Option<&Value>,
//     context: &ServerContext,
// ) -> Result<Response<Body>, Error> {
//     let params = match params {
//         Some(p) => p,
//         None => {
//             return Ok(admin_error_response(
//                 "Missing params",
//             ))
//         }
//     };

//     let new_price: f32 = match serde_json::from_value(params.clone()) {
//         Ok(p) => p,
//         Err(e) => {
//             return Ok(admin_error_response(
//                 &e.to_string(),
//             ))
//         }
//     };

//     // Access the server state
//     let mut server_state = context.state.lock().await;

//     // Remove the valid IPFS hashes from the server state's bundles
//     server_state.price_per_byte = new_price;

//     Ok(Response::builder()
//         .status(StatusCode::OK)
//         .body("Price successfully updated".into())
//         .unwrap())
// }

// fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
//     headers
//         .get("Token")
//         .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
// }

// pub async fn graphql_handler(
//     // headers: HeaderMap,
//     req: GraphQLRequest,
// ) -> GraphQLResponse {
//     let mut req = req.into_inner();
//     if let Some(token) = get_token_from_headers(&headers) {
//         req = req.data(token);
//     }
//     schema.execute(req).await.into()
// }
