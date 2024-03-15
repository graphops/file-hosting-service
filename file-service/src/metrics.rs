use autometrics::encode_global_metrics;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use axum::Server;
use once_cell::sync::Lazy;
use prometheus::{core::Collector, Registry};
use prometheus::{HistogramOpts, HistogramVec, IntGaugeVec, Opts};

use crate::config::ServerArgs;

use tracing::debug;

// Response Time: response time for file requests.
#[allow(dead_code)]
pub static RESPONSE_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    let m = HistogramVec::new(
        HistogramOpts::new(
            "response_time",
            "Response time for file requests in microseconds",
        )
        .namespace("file_service"),
        &["file_id"],
    )
    .expect("Failed to create response_time timer");
    prometheus::register(Box::new(m.clone())).expect("Failed to register response_time timer");
    m
});

// Number of bytes transferred per request per file
#[allow(dead_code)]
pub static TRANSFERRED_BYTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    let m = IntGaugeVec::new(
        Opts::new(
            "transferred_bytes",
            "Number of bytes transferred per request",
        )
        .namespace("file_service"),
        &["file_id"],
    )
    .expect("Failed to create transferred_bytes gauge");
    prometheus::register(Box::new(m.clone())).expect("Failed to register transferred_byes gauge");
    m
});

#[allow(dead_code)]
pub static REGISTRY: Lazy<prometheus::Registry> = Lazy::new(prometheus::Registry::new);

#[allow(dead_code)]
pub fn register_metrics(registry: &Registry, metrics: Vec<Box<dyn Collector>>) {
    for metric in metrics {
        registry.register(metric).expect("Cannot register metrics");
        debug!("registered metric");
    }
}

#[allow(dead_code)]
pub fn start_metrics() {
    register_metrics(
        &REGISTRY,
        vec![
            Box::new(RESPONSE_TIME.clone()),
            Box::new(TRANSFERRED_BYTES.clone()),
        ],
    );
}

/// This handler serializes the metrics into a string for Prometheus to scrape
#[allow(dead_code)]
pub async fn get_metrics() -> (StatusCode, String) {
    match encode_global_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:?}")),
    }
}

pub fn serve_metrics(config: &ServerArgs) {
    start_metrics();
    if config.metrics_host_and_port.is_none() {
        return;
    };
    // Set up the exporter to collect metrics
    let app = Router::new().route("/metrics", get(get_metrics));
    let metrics_addr = config.metrics_host_and_port.unwrap();
    tokio::spawn(async move {
        Server::bind(&metrics_addr)
            .serve(app.into_make_service())
            .await
            .expect("Failed to initialize admin server")
    });
}
