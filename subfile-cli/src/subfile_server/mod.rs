// #![cfg(feature = "acceptor")]
use anyhow::anyhow;
use hyper::service::{make_service_fn, service_fn};
use serde::Serialize;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::{validate_subfile_entries, ServerArgs};
use crate::ipfs::IpfsClient;
use crate::subfile_reader::read_subfile;
use crate::subfile_server::util::package_version;
use crate::types::Subfile;
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::{Body, Request, Response, StatusCode, header::RANGE};

use self::range::{parse_range_header, serve_file, serve_file_range};
use self::util::PackageVersion;

pub mod range;
pub mod util;

// Define a struct for the server state
#[derive(Debug)]
pub struct ServerState {
    pub subfiles: HashMap<String, Subfile>, // Keyed by IPFS hash
    pub release: PackageVersion,
}

pub type ServerContext = Arc<Mutex<ServerState>>;

pub async fn init_server(client: &IpfsClient, config: ServerArgs) {
    let port = config.port;
    let addr = format!("{}:{}", config.host, port)
        .parse()
        .expect("Invalid address");

    let state = initialize_subfile_server_context(client, config)
        .await
        .unwrap();

    // Create hyper server routes
    let make_svc = make_service_fn(|_| {
        let state = state.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
    });

    // TODO: add these to configs
    // let certs = load_certs("path/to/cert.pem").expect("Failed to load certs");
    // let key = load_private_key("path/to/key.pem").expect("Failed to load private key");

    // let tls_cfg = {
    //     let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    //     cfg.set_single_cert(certs, key).expect("Invalid key or certificate");
    //     Arc::new(cfg)
    // };

    // let acceptor = TlsAcceptor::from(tls_cfg);
    // let server = Server::builder(hyper::server::accept::from_stream(acceptor.accept_stream()))
    //     .serve(make_svc);
    let server = hyper::server::Server::bind(&addr).serve(make_svc);

    tracing::info!("Server listening on https://{}", addr);

    if let Err(e) = server.await {
        tracing::error!("server error: {}", e);
    }
}


/// Function to initialize the subfile server
async fn initialize_subfile_server_context(
    client: &IpfsClient,
    config: ServerArgs,
) -> Result<ServerContext, anyhow::Error> {
    let subfile_entries = validate_subfile_entries(config.subfiles.clone())?;
    tracing::debug!(
        entries = tracing::field::debug(&subfile_entries),
        "Validated subfile entries"
    );

    // Add the file to the service availability endpoint
    // This would be part of your server state initialization
    let mut server_state = ServerState {
        subfiles: HashMap::new(),
        release: package_version()?,
    };

    // Fetch the file using IPFS client, should be verified
    for (ipfs_hash, local_path) in subfile_entries {
        let subfile = read_subfile(client, &ipfs_hash, local_path).await?;
        tracing::debug!(subfile = tracing::field::debug(&subfile), "Read subfile");
        server_state
            .subfiles
            .insert(subfile.ipfs_hash.clone(), subfile);
    }

    // Return the server state wrapped in an Arc for thread safety
    Ok(Arc::new(Mutex::new(server_state)))
}

/// Handle incoming requests by
pub async fn handle_request(
    req: Request<Body>,
    context: ServerContext,
) -> Result<Response<Body>, anyhow::Error> {
    match req.uri().path() {
        "/" => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("Ready to roll!".into())
            .unwrap()),
        "/operator" => {
            //TODO: Implement logic to return operator info
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body("TODO: Operator info".into())
                .unwrap())
        }
        "/status" => {
            //TODO: Implement logic to return availability status from server context subfiles
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body("TODO: Read server status".into())
                .unwrap())
        }
        "/health" => health().await,
        "/version" => version(&context).await,
        path if path.starts_with("/subfiles/id/") => {
            let id = path.trim_start_matches("/subfiles/id/");

            let context_ref = context.lock().await;
            let file_path = match context_ref.subfiles.get(id).map(|f| f.local_path.as_path()) {
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

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

/// Endpoint for server health
pub async fn health() -> Result<Response<Body>, anyhow::Error> {
    let health = Health { healthy: true };
    let health_json = serde_json::to_string(&health).map_err(|e| anyhow!(e.to_string()))?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(health_json))
        .unwrap())
}

/// Endpoint for package version
pub async fn version(context: &ServerContext) -> Result<Response<Body>, anyhow::Error> {
    let version = context.lock().await.release.version.clone();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(version))
        .unwrap())
}

/// Endpoint for status availability
pub async fn status(context: &ServerContext) -> Result<Response<Body>, anyhow::Error> {
    let subfile_mapping = context.lock().await.subfiles.clone();
    // TODO: check for local access

    let subfile_ipfses: Vec<String> = subfile_mapping.keys().into_iter().map(|i| i.to_owned()).collect::<Vec<String>>();
    let json = serde_json::to_string(&subfile_ipfses).map_err(|e| anyhow!(e.to_string()))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .unwrap())
}
