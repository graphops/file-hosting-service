// #![cfg(feature = "acceptor")]

use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use file_exchange::errors::Error;
use file_exchange::manifest::{
    ipfs::IpfsClient, manifest_fetcher::read_bundle, validate_bundle_entries, Bundle,
};

use crate::config::{Config, ServerArgs};
use crate::file_server::{admin::handle_admin_request, util::public_key};
// #![cfg(feature = "acceptor")]
// use hyper_rustls::TlsAcceptor;
use hyper::{Body, Request, Response, StatusCode};

pub mod admin;
pub mod range;
pub mod routes;
pub mod util;

// Define a struct for the server state
#[derive(Debug)]
pub struct ServerState {
    pub client: IpfsClient,
    pub operator_public_key: String,
    pub bundles: HashMap<String, Bundle>, // Keyed by IPFS hash
    pub release: util::PackageVersion,
    pub free_query_auth_token: Option<String>, // Add bearer prefix
    pub admin_auth_token: Option<String>,      // Add bearer prefix
    pub price_per_byte: f32,
}

pub type ServerContext = Arc<Mutex<ServerState>>;

pub async fn init_server(config: Config) {
    let client = if let Ok(client) = IpfsClient::new(&config.ipfs_gateway) {
        client
    } else {
        IpfsClient::localhost()
    };

    let config = config.server;

    let port = config.port;
    let addr = format!("{}:{}", config.host, port)
        .parse()
        .expect("Invalid address");

    let state = initialize_server_context(&client, config)
        .await
        .expect("Failed to initiate bundle server");

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

/// Function to initialize the file hosting server
async fn initialize_server_context(
    client: &IpfsClient,
    config: ServerArgs,
) -> Result<ServerContext, Error> {
    tracing::debug!(
        config = tracing::field::debug(&config),
        "Initializing server context"
    );

    let bundle_entries = validate_bundle_entries(config.bundles.clone())?;
    tracing::debug!(
        entries = tracing::field::debug(&bundle_entries),
        "Validated bundle entries"
    );

    let free_query_auth_token = config
        .free_query_auth_token
        .map(|token| format!("Bearer {}", token));
    let admin_auth_token = config
        .admin_auth_token
        .map(|token| format!("Bearer {}", token));

    build_info::build_info!(fn build_info);
    // Add the file to the service availability endpoint
    // This would be part of your server state initialization
    let mut server_state = ServerState {
        client: client.clone(),
        bundles: HashMap::new(),
        release: util::PackageVersion::from(build_info()),
        free_query_auth_token,
        admin_auth_token,
        operator_public_key: public_key(&config.mnemonic)
            .expect("Failed to initiate with operator wallet"),
        price_per_byte: config.price_per_byte,
    };

    // Fetch the file using IPFS client
    for (ipfs_hash, local_path) in bundle_entries {
        let bundle = read_bundle(&server_state.client, &ipfs_hash, local_path).await?;
        let _ = bundle.validate_local_bundle();

        server_state
            .bundles
            .insert(bundle.ipfs_hash.clone(), bundle);
    }

    // Return the server state wrapped in an Arc for thread safety
    Ok(Arc::new(Mutex::new(server_state)))
}

/// Handle incoming requests by
pub async fn handle_request(
    req: Request<Body>,
    context: ServerContext,
) -> Result<Response<Body>, Error> {
    tracing::trace!("Received request");
    match req.uri().path() {
        "/" => Ok(Response::builder()
            .status(StatusCode::OK)
            .body("Ready to roll!".into())
            .unwrap()),
        "/operator" => routes::operator_info(&context).await,
        "/status" => routes::status(&context).await,
        "/health" => routes::health().await,
        "/version" => routes::version(&context).await,
        "/cost" => routes::cost(&context).await,
        "/admin" => handle_admin_request(req, &context).await,
        //TODO: consider routing through file level IPFS
        path if path.starts_with("/bundles/id/") => {
            routes::file_service(path, &req, &context).await
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Route not found".into())
            .unwrap()),
    }
}

/// Create an error response
pub fn create_error_response(msg: &str, status_code: StatusCode) -> Response<Body> {
    let body = json!({ "error": msg }).to_string();
    Response::builder()
        .status(status_code)
        .body(body.into())
        .unwrap()
}
