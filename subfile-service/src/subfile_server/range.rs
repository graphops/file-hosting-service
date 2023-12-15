// #![cfg(feature = "acceptor")]
use hyper::header::{CONTENT_LENGTH, CONTENT_RANGE};
use hyper::{Body, Response, StatusCode};

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

use std::path::Path;

use subfile_exchange::errors::{Error, ServerError};

// Function to parse the Range header and return the start and end bytes
pub fn parse_range_header(range_header: &hyper::header::HeaderValue) -> Result<(u64, u64), Error> {
    let range_str = range_header
        .to_str()
        .map_err(|e| Error::InvalidRange(format!("Invalid Range header: {}", e)))?;

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
        .parse::<u64>()
        .map_err(|e| Error::InvalidRange(format!("Invalid start range: {}", e)))?;
    let end = ranges[1]
        .parse::<u64>()
        .map_err(|e| Error::InvalidRange(format!("Invalid end range: {}", e)))?;

    Ok((start, end))
}

pub async fn serve_file_range(
    file_path: &Path,
    (start, end): (u64, u64),
) -> Result<Response<Body>, Error> {
    tracing::debug!(
        file_path = tracing::field::debug(&file_path),
        start_byte = tracing::field::debug(&start),
        end_byte = tracing::field::debug(&end),
        "Serve file range"
    );
    //TODO: Map the subfile_id to a file path, use server state for the file_map
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Cannot access file: {:#?}", e.to_string()).into())
                .unwrap());
        }
    };

    let file_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Cannot get file metadata: {:#?}", e.to_string()).into())
                .unwrap())
        }
    };

    tracing::debug!(start, end, file_size, "Range validity check");
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

    match file.seek(SeekFrom::Start(start)) {
        Ok(_) => {
            tracing::trace!("File seek to start at {:#?}", start)
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to seek file start: {:#?}", e.to_string()).into())
                .unwrap())
        }
    }

    let mut buffer = vec![0; length as usize];
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to react exact bytes: {:#?}", e.to_string()).into())
                .unwrap())
        }
    }

    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(
            CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                start,
                end,
                file.metadata().map_err(Error::FileIOError)?.len()
            ),
        )
        .header(CONTENT_LENGTH, length.to_string())
        .body(Body::from(buffer))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}

pub async fn serve_file(file_path: &Path) -> Result<Response<Body>, Error> {
    // If no Range header is present, serve the entire file
    let file = match fs::read(file_path) {
        Ok(f) => f,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Cannot read file".into())
                .unwrap())
        }
    };

    Response::builder()
        .body(Body::from(file))
        .map_err(|e| Error::ServerError(ServerError::BuildResponseError(e.to_string())))
}
