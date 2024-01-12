use clap::arg;
use clap::{command, Args, Parser};
use serde::{Deserialize, Serialize};
use std::fmt;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;

#[derive(Clone, Debug, Parser, Serialize, Deserialize)]
#[command(
    name = "file-service",
    about = "Indexer file hosting service",
    author = "hopeyen"
)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Config {
    #[command(flatten)]
    pub server: ServerArgs,
    #[arg(
        long,
        value_name = "IPFS_GATEWAY_URL",
        default_value = "https://ipfs.network.thegraph.com",
        env = "IPFS_GATEWAY_URL",
        help = "IPFS gateway to interact with"
    )]
    pub ipfs_gateway: String,
    #[arg(
        long,
        value_name = "LOG_FORMAT",
        env = "LOG_FORMAT",
        help = "Support logging formats: pretty, json, full, compact",
        long_help = "pretty: verbose and human readable; json: not verbose and parsable; compact:  not verbose and not parsable; full: verbose and not parsible",
        default_value = "pretty"
    )]
    pub log_format: LogFormat,
}

impl Config {
    /// Parse config arguments
    pub fn args() -> Self {
        let config = Config::parse();
        // Enables tracing under RUST_LOG variable
        init_tracing(config.log_format.to_string()).expect("Could not set up global default subscriber for logger, check environmental variable `RUST_LOG` or the CLI input `log-level`");
        config
    }
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct ServerArgs {
    #[arg(
        long,
        value_name = "HOST",
        default_value = "127.0.0.1",
        env = "HOST",
        help = "File server host"
    )]
    pub host: String,
    #[arg(
        long,
        value_name = "PORT",
        default_value = "5678",
        env = "PORT",
        help = "File server port"
    )]
    pub port: usize,
    // Taking from config right now, later can read from DB table for managing server states
    #[arg(
        long,
        value_name = "BUNDLES",
        env = "BUNDLES",
        value_delimiter = ',',
        help = "Comma separated list of IPFS hashes and local location of the bundles to serve upon start-up; format: [ipfs_hash:local_path]"
    )]
    pub bundles: Vec<String>,
    #[clap(
        long,
        value_name = "free-query-auth-token",
        env = "FREE_QUERY_AUTH_TOKEN",
        help = "Auth token that clients can use to query for free"
    )]
    pub free_query_auth_token: Option<String>,
    #[clap(
        long,
        value_name = "admin-auth-token",
        env = "ADMIN_AUTH_TOKEN",
        help = "Admin Auth token for server management"
    )]
    pub admin_auth_token: Option<String>,
    #[clap(
        long,
        value_name = "mnemonic",
        env = "MNEMONIC",
        help = "Mnemonic for the operator wallet"
    )]
    pub mnemonic: String,
    //TODO: More complex price management
    #[arg(
        long,
        value_name = "PRICE_PER_BYTE",
        default_value = "1",
        env = "PRICE_PER_BYTE",
        help = "Price per byte; price do not currently have a unit, perhaps use DAI or GRT, refer to TAP"
    )]
    pub price_per_byte: f32,
}

/// Sets up tracing, allows log level to be set from the environment variables
pub fn init_tracing(format: String) -> Result<(), SetGlobalDefaultError> {
    let filter = EnvFilter::from_default_env();

    let subscriber_builder: tracing_subscriber::fmt::SubscriberBuilder<
        tracing_subscriber::fmt::format::DefaultFields,
        tracing_subscriber::fmt::format::Format,
        EnvFilter,
    > = FmtSubscriber::builder().with_env_filter(filter);

    match format.as_str() {
        "json" => tracing::subscriber::set_global_default(subscriber_builder.json().finish()),
        "full" => tracing::subscriber::set_global_default(subscriber_builder.finish()),
        "compact" => tracing::subscriber::set_global_default(subscriber_builder.compact().finish()),
        _ => tracing::subscriber::set_global_default(
            subscriber_builder.with_ansi(true).pretty().finish(),
        ),
    }
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize, Default)]
pub enum LogFormat {
    Compact,
    #[default]
    Pretty,
    Json,
    Full,
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::Compact => write!(f, "compact"),
            LogFormat::Pretty => write!(f, "pretty"),
            LogFormat::Json => write!(f, "json"),
            LogFormat::Full => write!(f, "full"),
        }
    }
}
