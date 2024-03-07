use bytes::Bytes;

use reqwest::{header::HeaderName, Client};

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{errors::Error, manifest::file_hasher::verify_chunk};

#[derive(Debug, Clone)]
pub struct DownloadRangeRequest {
    pub receiver: String,
    pub query_endpoint: String,
    pub file_hash: String,
    pub start: u64,
    pub end: u64,
    pub chunk_hash: String,
    pub file: Arc<Mutex<File>>,
    pub max_retry: u64,
}

/// Make request to download a chunk and write it to the file in position
pub async fn download_chunk_and_write_to_file(
    http_client: &Client,
    request: DownloadRangeRequest,
    auth_header: (HeaderName, String),
) -> Result<Arc<Mutex<File>>, Error> {
    let mut attempts = 0;

    tracing::debug!(
        request = tracing::field::debug(&request),
        "Making a range request"
    );
    loop {
        // Make the range request to download the chunk
        match request_chunk(
            http_client,
            &request.query_endpoint,
            auth_header.clone(),
            &request.file_hash,
            request.start,
            request.end,
        )
        .await
        {
            Ok(data) => {
                if verify_chunk(&data, &request.chunk_hash) {
                    // Lock the file for writing
                    let mut file_lock = request.file.lock().await;
                    file_lock
                        .seek(SeekFrom::Start(request.start))
                        .map_err(Error::FileIOError)?;
                    file_lock.write_all(&data).map_err(Error::FileIOError)?;
                    drop(file_lock);
                    return Ok(request.file); // Successfully written the chunk, exit loop
                } else {
                    // Immediately return and blacklist the indexer when a chunk received is invalid
                    let msg = format!(
                        "Failed to validate received chunk: {}",
                        &request.query_endpoint
                    );
                    tracing::warn!(msg);
                    return Err(Error::ChunkInvalid(msg));
                }
            }
            Err(e) => tracing::error!("Chunk download error: {:?}", e),
        }

        attempts += 1;
        if attempts >= request.max_retry {
            return Err(Error::DataUnavailable(
                "Max retry attempts reached for chunk download".to_string(),
            ));
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Make range request for a file to the bundle server
pub async fn request_chunk(
    http_client: &Client,
    query_endpoint: &str,
    auth_header: (HeaderName, String),
    file_hash: &str,
    start: u64,
    end: u64,
) -> Result<Bytes, Error> {
    let range = format!("bytes={}-{}", start, end);

    // indexer framework enforced that only authorization header is effective.
    // we move file_hash and content-range to body, but consider requesting indexer-framework to be more flexible

    let req_body = serde_json::json!({
        "file-hash": file_hash,
        "content-range": range,
    }
    );

    tracing::debug!(query_endpoint, range, "Make range request");
    let response = http_client
        .post(query_endpoint)
        .header(auth_header.0, auth_header.1)
        .json(&req_body)
        .send()
        .await
        .map_err(Error::Request)?;

    // Check if the server supports range requests
    if response.status().is_success() {
        Ok(response.bytes().await.map_err(Error::Request)?)
    } else {
        let err_msg = format!(
            "Server does not support range requests or the request failed: {:#?}",
            tracing::field::debug(&response.status()),
        );
        tracing::error!(
            status = tracing::field::debug(&response.status()),
            headers = tracing::field::debug(&response.headers()),
            chunk = tracing::field::debug(&response),
            "Server does not support range requests or the request failed"
        );
        Err(Error::InvalidRange(err_msg))
    }
}
