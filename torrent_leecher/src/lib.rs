pub mod util;

use std::{
    collections::HashMap, default, net::SocketAddr, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration,
};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use crate::util::librqbit::{
    http_api::{ApiAddTorrentResponse, HttpApi},
    http_api_client::HttpApiClient,
    peer_connection::PeerConnectionOptions,
    session::{
        AddTorrentOptions, AddTorrentResponse, ListOnlyResponse, ManagedTorrentState, Session,
        SessionOptions,
    },
    spawn_utils::{spawn, BlockingSpawner},
};
use size_format::SizeFormatterBinary as SF;
use tracing::{debug, error, info, warn, trace};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Parser, Debug)]
#[command(version, author, about)]
pub struct Opts {
    /// The loglevel
    #[arg(value_enum, short = 'v')]
    pub log_level: Option<LogLevel>,

    /// The interval to poll trackers, e.g. 30s.
    /// Trackers send the refresh interval when we connect to them. Often this is
    /// pretty big, e.g. 30 minutes. This can force a certain value.
    #[arg(short = 'i', long = "tracker-refresh-interval", value_parser = parse_duration::parse)]
    pub force_tracker_interval: Option<Duration>,

    /// The listen address for HTTP API
    #[arg(long = "http-api-listen-addr", default_value = "127.0.0.1:3030")]
    pub http_api_listen_addr: SocketAddr,

    /// Set this flag if you want to use tokio's single threaded runtime.
    /// It MAY perform better, but the main purpose is easier debugging, as time
    /// profilers work better with this one.
    #[arg(short, long)]
    pub single_thread_runtime: bool,

    #[arg(long = "disable-dht")]
    pub disable_dht: bool,

    /// Set this to disable DHT reading and storing it's state.
    /// For now this is a useful workaround if you want to launch multiple rqbit instances,
    /// otherwise DHT port will conflict.
    #[arg(long = "disable-dht-persistence")]
    pub disable_dht_persistence: bool,

    /// The connect timeout, e.g. 1s, 1.5s, 100ms etc.
    #[arg(long = "peer-connect-timeout", value_parser = parse_duration::parse)]
    pub peer_connect_timeout: Option<Duration>,

    /// The connect timeout, e.g. 1s, 1.5s, 100ms etc.
    #[arg(long = "peer-read-write-timeout" , value_parser = parse_duration::parse)]
    pub peer_read_write_timeout: Option<Duration>,

    /// How many threads to spawn for the executor.
    #[arg(short = 't', long)]
    pub worker_threads: Option<usize>,

    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Parser, Default, Debug)]
pub struct ServerStartOptions {
    /// The output folder to write to. If not exists, it will be created.
    pub output_folder: String,
}

#[derive(Parser, Debug, Default)]
pub struct ServerOpts {
    #[clap(subcommand)]
    subcommand: ServerSubcommand,
}

#[derive(Parser, Debug)]
pub enum ServerSubcommand {
    Start(ServerStartOptions),
}

impl Default for ServerSubcommand {
    fn default() -> Self {
        ServerSubcommand::Start(ServerStartOptions::default())
    }
}

#[derive(Parser, Default, Debug)]
pub struct DownloadOpts {
    /// The filename or URL of the torrent. If URL, http/https/magnet are supported.
    pub torrent_path: Vec<String>,

    /// The output folder to write to. If not exists, it will be created.
    /// If not specified, would use the server's output folder. If there's no server
    /// running, this is required.
    #[arg(short = 'o', long)]
    pub output_folder: Option<String>,

    /// The sub folder within output folder to write to. Useful when you have
    /// a server running with output_folder configured, and don't want to specify
    /// the full path every time.
    #[arg(short = 's', long)]
    pub sub_folder: Option<String>,

    /// If set, only the file whose filename matching this regex will
    /// be downloaded
    #[arg(short = 'r', long = "filename-re")]
    pub only_files_matching_regex: Option<String>,

    /// Only list the torrent metadata contents, don't do anything else.
    #[arg(short, long)]
    pub list: bool,

    /// Set if you are ok to write on top of existing files
    #[arg(long)]
    pub overwrite: bool,

    /// Exit the program once the torrents complete download.
    #[arg(short = 'e', long, default_value = "true")]
    pub exit_on_finish: bool,
}

// server start
// download [--connect-to-existing] --output-folder(required) [file1] [file2]

#[derive(Parser, Debug)]
pub enum SubCommand {
    Server(ServerOpts),
    Download(DownloadOpts),
}

impl Default for SubCommand {
    fn default() -> Self {
        SubCommand::Download(DownloadOpts::default())
    }
}

fn init_logging(opts: &Opts) {
    if std::env::var_os("RUST_LOG").is_none() {
        match opts.log_level.as_ref() {
            Some(level) => {
                let level_str = match level {
                    LogLevel::Trace => "trace",
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warn => "warn",
                    LogLevel::Error => "error",
                };
                std::env::set_var("RUST_LOG", level_str);
            }
            None => {
                std::env::set_var("RUST_LOG", "info");
            }
        };
    }
    pretty_env_logger::init();
}

pub fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    init_logging(&opts);

    let (mut rt_builder, spawner) = match opts.single_thread_runtime {
        true => (
            tokio::runtime::Builder::new_current_thread(),
            BlockingSpawner::new(false),
        ),
        false => (
            {
                let mut b = tokio::runtime::Builder::new_multi_thread();
                if let Some(e) = opts.worker_threads {
                    b.worker_threads(e);
                }
                b
            },
            BlockingSpawner::new(true),
        ),
    };

    let rt = rt_builder
        .enable_time()
        .enable_io()
        // the default is 512, it can get out of hand, as this program is CPU-bound on
        // hash checking.
        // note: we aren't using spawn_blocking() anymore, so this doesn't apply,
        // however I'm still messing around, so in case we do, let's block the number of
        // spawned threads.
        .max_blocking_threads(8)
        .build()?;

    rt.block_on(async_main(opts, spawner))
}

pub async fn async_main(opts: Opts, spawner: BlockingSpawner) -> anyhow::Result<()> {
    let sopts = SessionOptions {
        disable_dht: opts.disable_dht,
        disable_dht_persistence: opts.disable_dht_persistence,
        dht_config: None,
        peer_id: None,
        peer_opts: Some(PeerConnectionOptions {
            connect_timeout: opts.peer_connect_timeout,
            read_write_timeout: opts.peer_read_write_timeout,
            ..Default::default()
        }),
    };
    let has_non_100 = |m: &HashMap<usize, f64>| m.values().any(|&v| v != 100f64);

    let stats_printer = |session: Arc<Session>| async move {
        let all_done = Arc::new(AtomicBool::new(false));
        loop {
            info!("Torrent session running");
            session.with_torrents(|torrents| {
                let mut torrent_finish_statuses = HashMap::new();
                for (idx, torrent) in torrents.iter().enumerate() {    
                    match &torrent.state {
                        ManagedTorrentState::Initializing => {
                            torrent_finish_statuses.insert(idx, 0f64);
                            // info!("[{}] initializing", idx);
                        },
                        ManagedTorrentState::Running(handle) => {
                            let peer_stats = handle.torrent_state().peer_stats_snapshot();
                            let stats = handle.torrent_state().stats_snapshot();
                            let speed = handle.speed_estimator();
                            let total = stats.total_bytes;
                            let progress = stats.total_bytes - stats.remaining_bytes;
                            let downloaded_pct = if stats.remaining_bytes == 0 {
                                info!(torrent_idx = idx, 
                                    "Download finished");
                                100f64
                            } else {
                                (progress as f64 / total as f64) * 100f64
                            };
                            torrent_finish_statuses.insert(idx, downloaded_pct);
                            debug!(
                                "[{}]: {:.2}% ({:.2}), down speed {:.2} MiB/s, fetched {}, remaining {:.2} of {:.2}, uploaded {:.2}, peers: {{live: {}, connecting: {}, queued: {}, seen: {}}}",
                                idx,
                                downloaded_pct,
                                SF::new(progress),
                                speed.download_mbps(),
                                SF::new(stats.fetched_bytes),
                                SF::new(stats.remaining_bytes),
                                SF::new(total),
                                SF::new(stats.uploaded_bytes),
                                peer_stats.live,
                                peer_stats.connecting,
                                peer_stats.queued,
                                peer_stats.seen,
                            );
                            
                        },
                    }
                }
                info!(statuses = tracing::field::debug(&torrent_finish_statuses), "torrent status");
                if !torrent_finish_statuses.is_empty() && !has_non_100(&torrent_finish_statuses) {
                    info!("All done! (¬‿¬)");
                    all_done.store(true, Ordering::SeqCst);
                }
            });
            
            if all_done.load(Ordering::SeqCst) {
                info!("Exiting the session");
                return Ok(())
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    match &opts.subcommand {
        SubCommand::Server(server_opts) => match &server_opts.subcommand {
            ServerSubcommand::Start(start_opts) => {
                let session = Arc::new(
                    Session::new_with_opts(
                        PathBuf::from(&start_opts.output_folder),
                        spawner,
                        sopts,
                    )
                    .await
                    .context("error initializing rqbit session")?,
                );
                spawn("Stats printer", stats_printer(session.clone()));
                let http_api = HttpApi::new(session);
                let http_api_listen_addr = opts.http_api_listen_addr;
                http_api
                    .make_http_api_and_run(http_api_listen_addr)
                    .await
                    .context("error starting HTTP API")
            }
        },
        SubCommand::Download(download_opts) => {
            if download_opts.torrent_path.is_empty() {
                anyhow::bail!("you must provide at least one URL to download")
            }
            let http_api_url = format!("http://{}", opts.http_api_listen_addr);
            let client = HttpApiClient::new(&http_api_url)?;
            let torrent_opts = AddTorrentOptions {
                only_files_regex: download_opts.only_files_matching_regex.clone(),
                overwrite: download_opts.overwrite,
                list_only: download_opts.list,
                force_tracker_interval: opts.force_tracker_interval,
                output_folder: download_opts.output_folder.clone(),
                sub_folder: download_opts.sub_folder.clone(),
                ..Default::default()
            };
            let connect_to_existing = match client.validate_rqbit_server().await {
                Ok(_) => {
                    info!(api = tracing::field::debug(client.base_url()), "Connected to HTTP API, will call it instead of downloading within this process");
                    true
                }
                Err(err) => {
                    warn!(
                        api = tracing::field::debug(client.base_url()),
                        err = err.to_string(),
                        "Error checking HTTP API"
                    );
                    false
                }
            };
            if connect_to_existing {
                for torrent_url in &download_opts.torrent_path {
                    match client
                        .add_torrent(torrent_url, Some(torrent_opts.clone()))
                        .await
                    {
                        Ok(ApiAddTorrentResponse { id, details }) => {
                            if let Some(id) = id {
                                info!("{} added to the server with index {}. Query {}/torrents/{}/(stats/haves) for details", details.info_hash, id, http_api_url, id)
                            }
                            for file in details.files {
                                info!(
                                    "file {:?}, size {}{}",
                                    file.name,
                                    SF::new(file.length),
                                    if file.included { "" } else { ", will skip" }
                                )
                            }
                        }
                        Err(err) => warn!(torrent_url, err = err.to_string(), "error adding"),
                    }
                }
                Ok(())
            } else {
                let session = Arc::new(
                    Session::new_with_opts(
                        download_opts
                            .output_folder
                            .as_ref()
                            .map(PathBuf::from)
                            .context(
                                "output_folder is required if can't connect to an existing server",
                            )?,
                        spawner,
                        sopts,
                    )
                    .await
                    .context("error initializing rqbit session")?,
                );
                spawn("Stats printer", stats_printer(session.clone()));
                let http_api = HttpApi::new(session.clone());
                let http_api_listen_addr = opts.http_api_listen_addr;
                spawn(
                    "HTTP API",
                    http_api.clone().make_http_api_and_run(http_api_listen_addr),
                );

                let mut added = false;

                let mut handles = Vec::new();

                for path in &download_opts.torrent_path {
                    let handle = match session.add_torrent(path, Some(torrent_opts.clone())).await {
                        Ok(v) => match v {
                            AddTorrentResponse::AlreadyManaged(handle) => {
                                info!(
                                    "torrent {:?} is already managed, downloaded to {:?}",
                                    handle.info_hash, handle.output_folder
                                );
                                continue;
                            }
                            AddTorrentResponse::ListOnly(ListOnlyResponse {
                                info_hash: _,
                                info,
                                only_files,
                            }) => {
                                for (idx, (filename, len)) in
                                    info.iter_filenames_and_lengths()?.enumerate()
                                {
                                    let included = match &only_files {
                                        Some(files) => files.contains(&idx),
                                        None => true,
                                    };
                                    info!(
                                        "File {}, size {}{}",
                                        filename.to_string()?,
                                        SF::new(len),
                                        if included { "" } else { ", will skip" }
                                    )
                                }
                                continue;
                            }
                            AddTorrentResponse::Added(handle) => {
                                added = true;
                                handle
                            }
                        },
                        Err(err) => {
                            error!("error adding {:?}: {:?}", &path, err);
                            continue;
                        }
                    };

                    http_api.add_torrent_handle(handle.clone());
                    handles.push(handle);
                }

                if download_opts.list {
                    Ok(())
                } else if added {
                    // if download_opts.exit_on_finish {
                        let results = futures::future::join_all(
                            handles.iter().map(|h| h.wait_until_completed()),
                        );
                        results.await;
                        info!("All downloads completed, exiting");
                        Ok(())
                    // } else {
                    //     // Sleep forever.
                    //     loop {
                    //         tokio::time::sleep(Duration::from_secs(60)).await;
                    //     }
                    // }
                } else {
                    anyhow::bail!("no torrents were added")
                }
            }
        }
    }
}
