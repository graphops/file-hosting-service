use clap::{arg, Args, Parser};
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

use figment::{
    providers::{Format, Toml},
    Figment,
};
use indexer_common::indexer_service::http::IndexerServiceConfig;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, value_name = "FILE")]
    pub config: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub common: IndexerServiceConfig,
    pub server: ServerArgs,
}

impl Config {
    pub fn load(filename: &PathBuf) -> Result<Self, figment::Error> {
        Figment::new().merge(Toml::file(filename)).extract()
    }
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct ServerArgs {
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
        value_name = "admin-auth-token",
        env = "ADMIN_AUTH_TOKEN",
        help = "Admin Auth token for server management"
    )]
    pub admin_auth_token: Option<String>,
    //TODO: More complex price management
    #[arg(
        long,
        value_name = "PRICE_PER_BYTE",
        default_value = "1",
        env = "PRICE_PER_BYTE",
        help = "Price per byte; price do not currently have a unit, perhaps use DAI or GRT, refer to TAP"
    )]
    pub price_per_byte: f32,
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
