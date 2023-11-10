// #![cfg(feature = "acceptor")]
use anyhow::anyhow;
use hyper::service::{make_service_fn, service_fn};

use std::collections::HashMap;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;

use crate::config::ServerArgs;
use crate::ipfs::IpfsClient;
use crate::subfile_reader::read_subfile;
use crate::types::Subfile;
// use hyper_rustls::TlsAcceptor;
use range::*;

pub mod range;

// Define a struct for the server state
pub struct ServerState {
    subfiles: HashMap<String, Subfile>, // Keyed by IPFS hash
}

pub type ServerContext = Arc<Mutex<ServerState>>;

pub async fn init_server(client: &IpfsClient, server_config: ServerArgs) {
    let port = server_config.port;
    let addr = format!("{}:{}", server_config.host, port)
        .parse()
        .expect("Invalid address");

    //TODO: add to configs
    let state = initialize_subfile_service(
        client,
        "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ",
        PathBuf::from("./example-file/example0017686312.dbin"),
    )
    .await
    .unwrap();

    // Create a hyper server
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

// // Custom echo service, handling two different routes and a
// // catch-all 404 responder.
// async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     let mut response = Response::new(Body::empty());
//     match (req.method(), req.uri().path()) {
//         // Help route.
//         (&Method::GET, "/") => {
//             *response.body_mut() = Body::from("Try POST /echo\n");
//         }
//         // Echo service route.
//         (&Method::POST, "/echo") => {
//             *response.body_mut() = req.into_body();
//         }
//         // Catch-all 404.
//         _ => {
//             *response.status_mut() = StatusCode::NOT_FOUND;
//         }
//     };
//     Ok(response)
// }

// Load public certificate from file.
#[allow(unused)]
fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>, anyhow::Error> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| anyhow!(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|e| anyhow!(format!("failed to load certificate: {:#?}", e)))?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

// Load private key from file.
#[allow(unused)]
fn load_private_key(filename: &str) -> Result<rustls::PrivateKey, anyhow::Error> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| anyhow!(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|e| anyhow!("failed to load private key: {:#?}", e))?;
    if keys.len() != 1 {
        return Err(anyhow!("expected a single private key"));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}

/// Function to initialize the subfile server
//TODO: Take in a vector of initial subfiles
async fn initialize_subfile_service(
    client: &IpfsClient,
    ipfs_hash: &str,
    local_path: PathBuf,
) -> Result<ServerContext, anyhow::Error> {
    //TODO: vectorize initial subfiles -> server_state subfiles hashmap
    // Fetch the file using IPFS client, should be verified
    let subfile = read_subfile(client, ipfs_hash, local_path).await?;

    // Add the file to the service availability endpoint
    // This would be part of your server state initialization
    let mut server_state = ServerState {
        subfiles: HashMap::new(),
    };
    server_state
        .subfiles
        .insert(subfile.ipfs_hash.clone(), subfile);

    // Return the server state wrapped in an Arc for thread safety
    Ok(Arc::new(Mutex::new(server_state)))
}
