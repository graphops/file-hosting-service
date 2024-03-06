use file_exchange::manifest::local_file_system::Store;
// #![cfg(feature = "acceptor")]
use hyper::header::{CONTENT_LENGTH, CONTENT_RANGE};
use hyper::{Body, Response, StatusCode};

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use object_store::path::Path;

use serde_json::Value;

use file_exchange::errors::{Error, ServerError};

// Function to parse the Range header and return the start and end bytes
pub fn parse_range_header(range_header: &Value) -> Result<(usize, usize), Error> {
    let range_str = range_header
        .as_str()
        .ok_or(Error::InvalidRange("Range header Not found".to_string()))?;

    if !range_str.starts_with("bytes=") {
        return Err(Error::InvalidRange(
            "Range does not start with 'bytes='".to_string(),
        ));
    }

    let ranges: Vec<&str> = range_str["bytes=".len()..].split('-').collect();
    if ranges.len() != 2 {
        return Err(Error::InvalidRange(
            "Invalid Range header format".to_string(),
        ));
    }

    let start = ranges[0]
        .parse::<usize>()
        .map_err(|e| Error::InvalidRange(format!("Invalid start range: {}", e)))?;
    let end = ranges[1]
        .parse::<usize>()
        .map_err(|e| Error::InvalidRange(format!("Invalid end range: {}", e)))?;

    Ok((start, end))
}

pub async fn serve_file_range(
    store: Store,
    file_name: &str,
    file_prefix: &Path,
    (start, end): (usize, usize),
) -> Result<Response<Body>, Error> {
    tracing::debug!(
        file_name = tracing::field::debug(&file_name),
        file_prefix = tracing::field::debug(&file_prefix),
        start_byte = tracing::field::debug(&start),
        end_byte = tracing::field::debug(&end),
        "Serve file range"
    );

    let metadata = store.find_object(file_name, Some(file_prefix)).await.ok_or(Error::DataUnavailable(format!("Cannot find object {} with prefix {}", file_name, file_prefix)))?;

    let file_size = metadata.size;
    tracing::debug!(
        metadata = tracing::field::debug(&metadata),
        file_size = tracing::field::debug(&file_size),
        "Serve file range"
    );

    tracing::trace!(start, end, file_size, "Range validity check");
    if start >= file_size || end >= file_size {
        return Ok(Response::builder()
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .body(
                format!(
                    "Range ({:#?}, {:#?}) out of bound for file size {:#?}",
                    start, end, file_size
                )
                .into(),
            )
            .unwrap());
    }

    let length = end - start + 1;
    let range = std::ops::Range { start, end: start + length };
    let content = store.range_read(file_name, range).await?;

    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(
            CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                start,
                end,
                length
            ),
        )
        .header(CONTENT_LENGTH, length.to_string())
        .body(Body::from(content))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

pub async fn serve_file(store: Store, file_name: &str, file_path: &Path) -> Result<Response<Body>, Error> {
    // If no Range header is present, serve the entire file
    let mut file = store.read(file_name).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).map_err(|e| Error::FileIOError(e))?;
    Response::builder()
        .body(Body::from(contents))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}
