// #![cfg(feature = "acceptor")]
use hyper::{Body, Request, Response, StatusCode};

use hyper::header::{CONTENT_LENGTH, CONTENT_RANGE, RANGE};

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

use anyhow::anyhow;
use std::path::Path;

use super::ServerContext;

// Function to parse the Range header and return the start and end bytes
fn parse_range_header(
    range_header: &hyper::header::HeaderValue,
) -> Result<(u64, u64), anyhow::Error> {
    let range_str = range_header
        // .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "Range header missing"))?
        .to_str()
        .map_err(|_| anyhow!("Invalid Range header"))?;

    if !range_str.starts_with("bytes=") {
        return Err(anyhow!("Range does not start with 'bytes='"));
    }

    let ranges: Vec<&str> = range_str["bytes=".len()..].split('-').collect();
    if ranges.len() != 2 {
        return Err(anyhow!("Invalid Range header format"));
    }

    let start = ranges[0]
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid start range"))?;
    let end = ranges[1]
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid end range"))?;

    Ok((start, end))
}

// pub async fn handle_range_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     let mut response = Response::new(Body::empty());

//     match (req.method(), req.uri().path()) {
//         // Serve range requests for a specific file.
//         (&Method::GET, "/file") => {
//             let file_path = "path/to/your/file"; // Set the path to your file here

//             // Check for the Range header
//             if let Some(range_header) = req.headers().get(hyper::header::RANGE) {
//                 match parse_range_header(Some(range_header)) {
//                     Ok((start, end)) => {
//                         let mut file = match File::open(file_path) {
//                             Ok(f) => f,
//                             Err(_) => {
//                                 *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
//                                 return Ok(response);
//                             }
//                         };

//                         let file_size = match file.metadata() {
//                             Ok(metadata) => metadata.len(),
//                             Err(_) => {
//                                 *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
//                                 return Ok(response);
//                             }
//                         };

//                         if start >= file_size {
//                             *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
//                             return Ok(response);
//                         }

//                         let end = end.min(file_size - 1);
//                         let length = end - start + 1;

//                         let mut buffer = vec![0; length as usize];
//                         match file.seek(SeekFrom::Start(start)) {
//                             Ok(_) => {}
//                             Err(_) => {
//                                 *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
//                                 return Ok(response);
//                             }
//                         }

//                         match file.read_exact(&mut buffer) {
//                             Ok(_) => {}
//                             Err(_) => {
//                                 *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
//                                 return Ok(response);
//                             }
//                         }

//                         *response.status_mut() = StatusCode::PARTIAL_CONTENT;
//                         response.headers_mut().insert(CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size).parse().unwrap());
//                         response.headers_mut().insert(CONTENT_LENGTH, length.to_string().parse().unwrap());
//                         *response.body_mut() = Body::from(buffer);
//                     },
//                     Err(_) => {
//                         *response.status_mut() = StatusCode::BAD_REQUEST;
//                     }
//                 }
//             } else {
//                 // If no Range header is present, serve the entire file
//                 let file = match fs::read(file_path) {
//                     Ok(f) => f,
//                     Err(_) => {
//                         *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
//                         return Ok(response);
//                     }
//                 };

//                 response.headers_mut().insert(ACCEPT_RANGES, "bytes".parse().unwrap());
//                 *response.body_mut() = Body::from(file);
//             }
//         }
//         // ... existing routes ...
//         _ => {
//             *response.status_mut() = StatusCode::NOT_FOUND;
//         }
//     };
//     Ok(response)
// }

pub async fn handle_request(
    req: Request<Body>,
    state: ServerContext,
) -> Result<Response<Body>, anyhow::Error> {
    match req.uri().path() {
        path if path.starts_with("/subfiles/id/") => {
            let id = path.trim_start_matches("/subfiles/id/");

            //TODO: serverState get id for local path
            let context = state.lock().await;
            let file_path = match context.subfiles.get(id).map(|f| f.local_path.as_path()) {
                Some(path) => path,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body("File not found".into())
                        .unwrap())
                }
            };
            // TODO: Add auth token config
            // let auth_token = req.headers().get(AUTHORIZATION)
            //     .and_then(|hv| hv.to_str().ok())
            //     .unwrap_or("");

            // // Validate the auth token
            // if auth_token != "expected_token" {
            //     return Ok(Response::builder()
            //         .status(StatusCode::UNAUTHORIZED)
            //         .body("Invalid auth token".into())
            //         .unwrap());
            // }

            // Parse the range header to get the start and end bytes
            match req.headers().get(RANGE) {
                Some(r) => {
                    let range = parse_range_header(r)
                        .map_err(|e| anyhow!(format!("Failed to parse range header: {}", e)))?;
                    //TODO: validate receipt
                    serve_file_range(file_path, range).await
                }
                None => serve_file(file_path).await,
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .unwrap()),
    }
}

async fn serve_file_range(
    file_path: &Path,
    (start, end): (u64, u64),
) -> Result<Response<Body>, anyhow::Error> {
    //TODO: Map the subfile_id to a file path, use server state for the file_map
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Cannot access file".into())
                .unwrap());
        }
    };

    let file_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Cannot get file metadata".into())
                .unwrap())
        }
    };

    if start >= file_size || end >= file_size {
        return Ok(Response::builder()
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .body("Range out of bound".into())
            .unwrap());
    }

    let length = end - start + 1;

    match file.seek(SeekFrom::Start(start)) {
        Ok(_) => {}
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Start range out of bound".into())
                .unwrap())
        }
    }

    let mut buffer = vec![0; length as usize];
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Range out of bound".into())
                .unwrap())
        }
    }

    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(
            CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, file.metadata()?.len()),
        )
        // .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, length.to_string())
        .body(Body::from(buffer))
        .map_err(|e| anyhow!(format!("Failed to build response: {}", e)))
}

async fn serve_file(file_path: &Path) -> Result<Response<Body>, anyhow::Error> {
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
        .map_err(|e| anyhow!(format!("Failed to build response: {}", e)))
}
