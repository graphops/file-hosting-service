// #![cfg(feature = "acceptor")]
use hyper::{Body, Request, Response, StatusCode};

use std::fs::{self};

use anyhow::anyhow;
use std::path::Path;

use super::ServerContext;

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
            serve_file(file_path).await
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .unwrap()),
    }
}

async fn serve_file(file_path: &Path) -> Result<Response<Body>, anyhow::Error> {
    // If no Range header is present, serve the entire file
    let file = match fs::read(file_path) {
        Ok(f) => f,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Start range out of bound".into())
                .unwrap())
        }
    };

    Response::builder()
        .body(Body::from(file))
        .map_err(|e| anyhow!(format!("Failed to build response: {}", e)))
}
