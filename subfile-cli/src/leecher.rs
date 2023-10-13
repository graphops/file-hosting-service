use crate::config::Leecher;
use crate::ipfs::IpfsClient;
use std::net::SocketAddr;
use std::process::Command;
use torrent_leecher::util::librqbit::spawn_utils::BlockingSpawner;
use std::str::FromStr;
use std::time::Duration;
use tokio::task::{self, JoinHandle};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;
use tracing::info;

use crate::types::Subfile;
// extern crate rqbit;

// Fetch subfile yaml from IPFS
async fn fetch_subfile_from_ipfs(
    client: &IpfsClient,
    ipfs_hash: &str,
) -> Result<Subfile, anyhow::Error> {
    // Fetch the content from IPFS
    let timeout = Duration::from_secs(30);

    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // add jitter to delays
        .take(5); // limit to 5 retries

    let file_bytes = Retry::spawn(retry_strategy, || client.cat_all(ipfs_hash, timeout)).await?;

    let content: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8(file_bytes.to_vec())?)?;

    tracing::info!("Got yaml file content");

    let subfile = convert_to_subfile(content)?;

    Ok(subfile)
}

fn convert_to_subfile(value: serde_yaml::Value) -> Result<Subfile, anyhow::Error> {
    tracing::trace!(
        value = tracing::field::debug(&value),
        "Parse yaml value into a subfile"
    );
    let subfile: Subfile = serde_yaml::from_value(value)?;

    //TODO: verify that the magnet link will truly result in the target subfile
    Ok(subfile)
}

pub async fn leech(
    client: &IpfsClient,
    ipfs_hash: &str,
    out_dir: &str,
) -> Result<Subfile, anyhow::Error> {
    let subfile: Subfile = fetch_subfile_from_ipfs(client, ipfs_hash).await?;

    tracing::trace!(
        // subfile = tracing::field::debug(&subfile),
        magnet_link = tracing::field::debug(&subfile.magnet_link),
        "Grabbed subfile"
    );

    //TODO: subfile's magnet link doesn't work right now because there is no seeding
    //TODO: temporarily continuing with an available file
    //TODO: Request torrent tracker and download

    let _ = download_file(&subfile.magnet_link, out_dir).await;

    //TODO: Verify the file

    Ok(subfile)
}

// Run a rqbit command to download a file from torrent
async fn download_file(magnet_link: &str, out_dir: &str) -> std::io::Result<()> {
    // let download_link = magnet_link.to_string();
    //TODO: currently not able to seed the torrent file we are making, using a file with widely available peers
    //no uploading available atm
    let temp_link = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=wss%3A%2F%2Ftracker.btorrent.xyz&tr=wss%3A%2F%2Ftracker.fastcast.nz&tr=wss%3A%2F%2Ftracker.openwebtorrent.com&ws=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2F&xs=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2Fbig-buck-bunny.torrent";
    info!(
        subfile_torrent_link = magnet_link,
        temporary_link = temp_link,
        output_directory = out_dir,
        "Leeching started, this may take a while so just hold on"
    );

    // leech_default(temp_link, out_dir);
    // let output = task::spawn_blocking(|| {
    //     run_rqbit(leech_default(temp_link, out_dir))
    //     // Command::new("cargo")
    //     //     .arg("run")
    //     //     .arg("-p")
    //     //     .arg("rqbit")
    //     //     .arg("download")
    //     //     .arg("-o") // download target directory
    //     //     .arg(out_dir)
    //     //     .arg("-e") // exit on download finish
    //     //     .arg("true")
    //     //     .arg(download_link)
    //     //     .output()
    // })
    // .await?;

    let leecher_handle = run_rqbit(leech_default(temp_link, out_dir)).await;
    info!(leecher_handle = tracing::field::debug(&leecher_handle), "Spawn leech thread");
    match leecher_handle {
        Ok(handle) => {
            info!(handle = tracing::field::debug(&handle), "now await leech thread");
            _ = handle.await;
            info!("finished leech");
            // if output.status.success() {
            //     let stdout = String::from_utf8_lossy(&output.stdout);
            // } else {
            //     let stderr = String::from_utf8_lossy(&output.stderr);
            //     info!("Error: {}", stderr);
            // }
        }
        Err(e) => {
            info!("Failed to leech: {}", e);
        }
    }

    Ok(())
}

fn leech_default(magnet_link: &str, out_dir: &str) -> torrent_leecher::Opts {
    let mut leecher = torrent_leecher::DownloadOpts::default();
    leecher.torrent_path = vec![magnet_link.to_string()];
    // leecher.exit_on_finish = true; // set this to default
    leecher.output_folder = Some(out_dir.to_string());
    torrent_leecher::Opts {
        log_level: None,
        force_tracker_interval: None,
        http_api_listen_addr: SocketAddr::from_str("127.0.0.1:3030").expect("Addr parsing"),
        single_thread_runtime: false,
        disable_dht: false,
        disable_dht_persistence: true,
        peer_connect_timeout: None,
        peer_read_write_timeout: None,
        worker_threads: None,
        subcommand: torrent_leecher::SubCommand::Download(leecher),
    }
}

async fn run_rqbit(opts: torrent_leecher::Opts) -> Result<JoinHandle<Result<(), anyhow::Error>>, anyhow::Error>{
    let (_, spawner) = match opts.single_thread_runtime {
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

    Ok(tokio::spawn(torrent_leecher::async_main(opts, spawner)))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn test_client() -> IpfsClient {

//     }

//     #[test]
//     fn fetch_random_subgraph_yaml() {// https://ipfs.network.thegraph.com/api/v0/cat?arg=Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX
//         let mut config = test_config();

//         assert_eq!(&config.protocol_network().unwrap(), "goerli");

//         config.graph_stack.protocol_network = Some("arbitrum-one".to_string());
//         assert_eq!(&config.protocol_network().unwrap(), "arbitrum-one");
//     }
// }
