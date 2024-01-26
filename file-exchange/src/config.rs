use clap::{arg, ValueEnum};
use clap::{command, Args, Parser, Subcommand};
use ethers_core::types::U256;
use serde::{Deserialize, Serialize};
use std::fmt;

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;

use crate::util::parse_key;

#[derive(Clone, Debug, Parser, Serialize, Deserialize)]
#[command(
    name = "file-exchange",
    about = "A CLI for file hosting exchanges",
    author = "hopeyen"
)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub role: Role,
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

impl Cli {
    /// Parse config arguments
    pub fn args() -> Self {
        let config = Cli::parse();
        // Enables tracing under RUST_LOG variable
        init_tracing(config.log_format.to_string()).expect("Could not set up global default subscriber for logger, check environmental variable `RUST_LOG` or the CLI input `log-level`");
        config
    }
}

#[derive(Clone, Debug, Subcommand, Serialize, Deserialize)]
#[group(required = false, multiple = true)]
pub enum Role {
    Downloader(DownloaderArgs),
    Publisher(PublisherArgs),
    Wallet(WalletArgs),
}

/// Server enable payments through the staking contract,
/// assume indexer is already registered on the staking registry contract
///1. `allocate` - indexer address, Qm hash in bytes32, token amount, allocation_id, metadata: utils.hexlify(Array(32).fill(0)), allocation_id_proof
///2. `close_allocate` -allocationID: String, poi: BytesLike (0x0 32bytes)
///3. `close_allocate` and then `allocate`
/// receipt validation and storage is handled by the indexer-service framework
/// receipt redemption is handled by indexer-agent
///
/// Client payments - assume client signer is valid (should work without gateways)
///1. `deposit` - to a sender address and an amount
///2. `depositMany` - to Vec<sender address, an amount>
#[derive(Clone, Debug, Subcommand, Serialize, Deserialize)]
#[group(required = false, multiple = true)]
pub enum OnchainAction {
    Allocate(AllocateArgs),
    Unallocate(UnallocateArgs),
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct WalletArgs {
    #[clap(subcommand)]
    pub action: Option<OnchainAction>,
    #[clap(
        long,
        value_name = "KEY",
        value_parser = parse_key,
        env = "MNEMONIC",
        hide_env_values = true,
        help = "Mnemonic to the Indexer operator wallet (first address of the wallet is used",
    )]
    pub mnemonic: String,
    #[clap(
        long,
        value_name = "provider_url",
        env = "PROVIDER",
        help = "Blockchain provider endpoint"
    )]
    pub provider: String,
    #[clap(
        long,
        value_name = "verifier",
        env = "VERIFIER",
        help = "TAP verifier contract address"
    )]
    pub verifier: Option<String>,
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct DownloaderArgs {
    #[arg(
        long,
        value_name = "IPFS_HASH",
        env = "IPFS_HASH",
        help = "IPFS hash for the target bundle.yaml"
    )]
    pub ipfs_hash: String,
    #[arg(
        long,
        value_name = "GATEWAY_URL",
        env = "GATEWAY_URL",
        help = "Client pings the gateway for file discovery; TODO: currently gateway_url is used to ping local server url directly"
    )]
    pub gateway_url: Option<String>,
    #[clap(
        long,
        value_name = "mnemonic",
        env = "MNEMONIC",
        help = "Mnemonic for payment wallet"
    )]
    pub mnemonic: String,
    #[clap(
        long,
        value_name = "provider_url",
        env = "PROVIDER",
        help = "Blockchain provider endpoint"
    )]
    pub provider: String,
    #[clap(
        long,
        value_name = "verifier",
        env = "VERIFIER",
        help = "TAP verifier contract address"
    )]
    pub verifier: String,
    // Trust tracking should be done by the gateway/DHT
    #[arg(
        long,
        value_name = "INDEXER_ENDPOINTS",
        value_delimiter = ',',
        env = "INDEXER_ENDPOINTS",
        help = "A list of indexer endpoints to query data from"
    )]
    pub indexer_endpoints: Vec<String>,
    #[arg(
        long,
        value_name = "OUTPUT_DIR",
        default_value = "./example-download",
        env = "OUTPUT_DIR",
        help = "Output directory for target files"
    )]
    pub output_dir: String,
    #[clap(
        long,
        value_name = "free-query-auth-token",
        env = "FREE_QUERY_AUTH_TOKEN",
        help = "Auth token that to query for free"
    )]
    pub free_query_auth_token: Option<String>,

    #[arg(
        long,
        value_name = "MAX_RETRY",
        default_value = "10",
        env = "MAX_RETRY",
        help = "Maximum retry for each chunk"
    )]
    pub max_retry: u64,
}

/// Publisher takes the files, generate bundle manifest, and publish to IPFS
//TODO: a single command to publish a range of files
#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct PublisherArgs {
    #[arg(
        long,
        value_name = "YAML_STORE_DIR",
        env = "YAML_STORE_DIR",
        default_value = "./example-file/bundle.yaml",
        help = "Path to the directory to store the generated yaml file for bundle"
    )]
    pub yaml_store: String,

    #[arg(
        long,
        value_name = "READ_DIR",
        env = "READ_DIR",
        help = "Path to the file for publishing (perhaps change to a regex or a vec later)"
    )]
    pub read_dir: String,

    #[arg(
        long,
        value_name = "BUNDLE_NAME",
        env = "BUNDLE_NAME",
        help = "Name for the bundle (later this can be interactive)"
    )]
    pub bundle_name: String,

    #[arg(
        long,
        value_name = "FILE_NAMES",
        value_delimiter = ',',
        env = "FILE_NAMES",
        help = "Name for the files to be included in bundle (later this can be interactive)"
    )]
    pub file_names: Vec<String>,

    #[arg(
        long,
        value_name = "FILE_TYPE",
        value_enum,
        env = "FILE_TYPE",
        //TODO: use enum
        // value_parser = clap::value_parser!(FileType::from_str),
        help = "Type of the file (e.g., sql_snapshot, flatfiles)"
    )]
    pub file_type: String,

    #[arg(
        long,
        value_name = "FILE_VERSION",
        env = "FILE_VERSION",
        //TODO: use enum
        // value_parser = clap::value_parser!(FileType::from_str),
        help = "Bundle versioning"
    )]
    pub bundle_version: String,

    #[arg(
        long,
        value_name = "IDENTIFIER",
        env = "IDENTIFIER",
        help = "Identifier of the file given its type (chain-id for firehose flatfiles, subgraph deployment hash for subgraph snapshots)"
    )]
    pub identifier: Option<String>,

    #[arg(
        long,
        value_name = "CHUNK_SIZE",
        env = "CHUNK_SIZE",
        default_value = "1048576",
        help = "Chunk size in bytes to split files (Default: 1048576 bytes = 1MiB)"
    )]
    pub chunk_size: u64,

    #[arg(
        long,
        value_name = "START_BLOCK",
        env = "START_BLOCK",
        help = "Start block for flatfiles"
    )]
    pub start_block: Option<u64>,

    #[arg(
        long,
        value_name = "END_BLOCK",
        env = "END_BLOCK",
        help = "End block for sql snapshot or flatfiles"
    )]
    pub end_block: Option<u64>,

    #[arg(
        long,
        value_name = "PUBLISHER_URL",
        env = "PUBLISHER_URL",
        help = "Self promoting endpoint to record inside the bundle (TODO: can update to be a github repository link)"
    )]
    pub publisher_url: Option<String>,

    #[arg(
        long,
        value_name = "DESCRIPTION",
        env = "DESCRIPTION",
        default_value = "",
        help = "Describe bundle content"
    )]
    pub description: String,

    #[arg(
        long,
        value_name = "NETWORK",
        env = "NETWORK",
        default_value = "1",
        help = "Network represented in CCIP ID (Ethereum mainnet: 1, goerli: 5, arbitrum-one: 42161, sepolia: 58008"
    )]
    pub chain_id: String,
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct AllocateArgs {
    #[clap(
        long,
        value_name = "tokens",
        env = "TOKENS",
        help = "Token amount to allocate (in units of GRT)",
        value_parser = U256::from_dec_str,
    )]
    pub tokens: U256,
    #[clap(
        long,
        value_name = "deployment_ipfs",
        env = "DEPLOYMENT_IPFS",
        help = "Deployment IPFS hash to allocate"
    )]
    pub deployment_ipfs: String,
    #[clap(
        long,
        value_name = "epoch",
        env = "EPOCH",
        help = "Epoch field to generate unique allocation id (Should be auto-resolve through network query)"
    )]
    pub epoch: u64,
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct UnallocateArgs {
    #[clap(
        long,
        value_name = "allocation_id",
        env = "ALLOCATION_ID",
        help = "Deployment IPFS hash to unallocate"
    )]
    pub allocation_id: String,
}

#[allow(unused)]
#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, Default)]
pub enum FileType {
    #[default]
    SqlSnapshot,
    Flatfiles,
}

// impl FromStr for FileType {
//     type Err = &'static str;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "sql_snapshot" => Ok(FileType::SqlSnapshot),
//             "flatfiles" => Ok(FileType::Flatfiles),
//             _ => Err("Invalid file type"),
//         }
//     }
// }

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