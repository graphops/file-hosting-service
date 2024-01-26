use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use super::{FileServiceError, ServerContext};

/// Endpoint for status availability
pub async fn status(
    State(context): State<ServerContext>,
) -> Result<impl IntoResponse, FileServiceError> {
    // pub async fn status(context: &ServerContext) -> Result<impl IntoResponse, FileServiceError> {
    let bundle_mapping = context.state.lock().await.bundles.clone();
    let bundle_ipfses: Vec<String> = bundle_mapping
        .keys()
        .map(|i| i.to_owned())
        .collect::<Vec<String>>();
    tracing::debug!(
        deployment = tracing::field::debug(&bundle_ipfses),
        "Bundles in status"
    );
    serde_json::to_string(&bundle_ipfses)
        .map(|data| Json(json!({"data": data})))
        .map_err(|e| FileServiceError::StatusQueryError(e.into()))
}
