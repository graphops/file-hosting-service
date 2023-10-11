use clap::{command, Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;

#[derive(Clone, Debug, Parser, Serialize, Deserialize)]
#[clap(
    name = "subfile-exchange",
    about = "A CLI for subfile exchanges",
    author = "hopeyen"
)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub role: Role,
    #[clap(
        long,
        value_name = "IPFS_GATEWAY_URL",
        default_value = "https://ipfs.network.thegraph.com",
        env = "IPFS_GATEWAY_URL",
        help = "IPFS gateway to interact with"
    )]
    pub ipfs_gateway: String,
    #[clap(
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
    Leecher(Leecher),
    Seeder(Seeder),
    Tracker(Tracker),
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct Tracker {
    #[clap(
        long,
        value_name = "SERVER_HOST",
        env = "SERVER_HOST",
        help = "Tracker server host"
    )]
    pub server_host: String,
    #[clap(
        long,
        value_name = "SERVER_PORT",
        env = "SERVER_PORT",
        help = "Tracker server port"
    )]
    pub server_port: usize,
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct Leecher {
    #[clap(
        long,
        value_name = "IPFS_HASH",
        env = "IPFS_HASH",
        help = "IPFS hash for the target subfile.yaml"
    )]
    pub ipfs_hash: String,
}

#[derive(Clone, Debug, Args, Serialize, Deserialize, Default)]
#[group(required = false, multiple = true)]
pub struct Seeder {
    #[clap(
        long,
        value_name = "SUBFILE_SEEDS",
        env = "SUBFILE_SEEDS",
        help = "A vector of ipfs hashes to the subfiles to support seeding for"
        // The continuously running program should take the vector of the ipfs, and support seeding indicated by the subfiles specifications
    )]
    pub file_config: Vec<String>,
    //TODO: open this up to be an API so the program can run continuously
    //TODO: make this into a nested subcommand with SeedCreationArg struct
    #[clap(
        long,
        value_name = "FILE_PATH",
        env = "FILE_PATH",
        help = "Path to the file for seeding"
    )]
    pub file_path: String,

    #[clap(
        long,
        value_name = "FILE_TYPE",
        value_enum,
        env = "FILE_TYPE",
        //TODO: use enum
        // value_parser = clap::value_parser!(FileType::from_str),
        help = "Type of the file (e.g., sql_snapshot, flatfiles)"
    )]
    pub file_type: String,

    #[clap(
        long,
        value_name = "IDENTIFIER",
        env = "IDENTIFIER",
        help = "Identifier of the file given its type"
    )]
    pub identifier: String,

    #[clap(
        long,
        value_name = "START_BLOCK",
        env = "START_BLOCK",
        help = "Start block for flatfiles"
    )]
    pub start_block: Option<u64>,

    #[clap(
        long,
        value_name = "END_BLOCK",
        env = "END_BLOCK",
        help = "End block for sql snapshot or flatfiles"
    )]
    pub end_block: Option<u64>,
}

// #[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, Default)]
// pub enum FileType {
//     #[default]
//     SqlSnapshot,
//     Flatfiles,
// }

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
