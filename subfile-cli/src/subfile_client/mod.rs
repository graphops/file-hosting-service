// #![deny(warnings)]
// use hyper;

// use std::env;
// use std::io::{self, Write};
// use hyper::Client;
// use hyper::header::{Connection, Range, ByteRangeSpec};

use anyhow::anyhow;
use bytes::Bytes;
use reqwest::Client;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tracing::info;

use http::header::{AUTHORIZATION, CONTENT_RANGE, RANGE};

use crate::config::DownloaderArgs;
use crate::file_hasher::hash_chunk;
use crate::ipfs::IpfsClient;
use crate::publisher::FileMetaInfo;
use crate::subfile_reader::{fetch_chunk_file_from_ipfs, fetch_subfile_from_ipfs};

pub struct SubfileDownloader {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
    ipfs_hash: String,
    //TODO: currently direct ping to server_url -> decentralize to gateway_url
    server_url: String,
    indexer_endpoints: Vec<String>,
    output_dir: String,
    free_query_auth_token: Option<String>,
    // Other fields as needed
}

impl SubfileDownloader {
    pub fn new(ipfs_client: IpfsClient, args: DownloaderArgs) -> Self {
        SubfileDownloader {
            ipfs_client,
            //TODO: consider if more advanced config like
            // a proxy should be used for the client
            http_client: reqwest::Client::new(),
            ipfs_hash: args.ipfs_hash,
            //TODO: change for discovery
            server_url: args.gateway_url,
            indexer_endpoints: args.indexer_endpoints,
            output_dir: args.output_dir,
            free_query_auth_token: args.free_query_auth_token,
        }
    }

    /// Check the availability of a subfile, ideally this should go through a gateway/DHT
    /// but for now we ping an indexer endpoint directly, which is what a gateway
    /// would do in behave of the downloader
    /// Return a list of endpoints where the desired subfile is hosted
    //TODO: update once there's a gateway
    pub async fn check_availability(&self) -> Result<Vec<String>, anyhow::Error> {
        tracing::info!("Check availability");

        let mut endpoints = vec![];
        // Loop through indexer endpoints and query data availability
        //TODO: parallelize
        for url in &self.indexer_endpoints {
            // Ping availability endpoint with timeout
            let status_url = url.to_owned() + "/status";
            let response = self.http_client.get(&status_url).send().await?;

            // Check if the server supports range requests
            // tracing::info!(response = tracing::field::debug(&response), "got response");
            if response.status().is_success() {
                tracing::info!(status_url = &status_url, "Successful status response");
                if let Ok(files) = response.json::<Vec<String>>().await {
                    if files.contains(&self.ipfs_hash) {
                        endpoints.push(status_url);
                    }
                }
            } else {
                tracing::error!("Server does not support range requests or the request failed.");
                return Err(anyhow!("Range request failed"));
            }

            // return a vec of indexers
            //TODO: add indexer pricing to do basic byte prices matching
        }

        Ok(endpoints)
    }

    /// Read subfile manifiest and download the individual chunk files
    //TODO: update once there is payment
    pub async fn download_subfile(&self) -> Result<(), anyhow::Error> {
        // Read subfile from ipfs
        let subfile = fetch_subfile_from_ipfs(&self.ipfs_client, &self.ipfs_hash).await?;

        // Loop through chunk files for downloading
        let mut completed_files = vec![];
        for chunk_file in &subfile.files {
            let res = self.download_chunk_file(chunk_file).await;
            completed_files.push(res);
        }

        //TODO: retry for failed subfiles
        tracing::info!(
            completed_files = tracing::field::debug(&completed_files),
            "Chunk files download results"
        );
        Ok(())
    }

    /// Download a file by reading its chunk manifest
    //TODO: update once there is payment
    pub async fn download_chunk_file(
        &self,
        chunk_file_info: &FileMetaInfo,
    ) -> Result<(), anyhow::Error> {
        tracing::debug!(
            info = tracing::field::debug(&chunk_file_info),
            "Download chunk file"
        );
        // First read subfile manifest for a chunk file, maybe the chunk_file ipfs_hash has been passed in
        let chunk_file =
            fetch_chunk_file_from_ipfs(&self.ipfs_client, &chunk_file_info.hash).await?;

        // read chunk file for file meta info; piece_length, total_bytes
        // calculate num_pieces = total_bytes / piece_length
        // parallelize chunk request by indexing through (0, num_pieces)

        // make chunk request with appropriate byte range
        // (index * piece_length + min((index+1)*piece_length, total_bytes))
        // record the chunk and write with bytes precision

        // check data availability from gateway/indexer_endpoints
        //TODO: gateway ISA
        let query_endpoint = format!("{}{}", self.server_url.clone(), self.ipfs_hash.clone());
        // let query_endpoints = self.check_availability().await?;
        // Assuming the URL is something like "http://localhost:5678/subfiles/id/"

        // Open the output file
        let file = File::create(Path::new(
            &(self.output_dir.clone() + "/" + &chunk_file.file_name),
        ))
        .unwrap();
        let file = Arc::new(Mutex::new(file));

        // Calculate the ranges and spawn threads to download each chunk
        let chunk_size = chunk_file.chunk_size;
        let hashes = chunk_file.chunk_hashes.clone();
        let mut handles = Vec::new();
        for i in 0..(chunk_file.total_bytes / chunk_size + 1) {
            tracing::trace!(i, "Download chunk index");
            let start = i * chunk_size;
            let end = u64::min(start + chunk_size, chunk_file.total_bytes) - 1;
            let file_clone = Arc::clone(&file);
            //TODO: use one of the checked availability endpoints
            let url = query_endpoint.clone();
            let client = self.http_client.clone();
            let auth_token = self.free_query_auth_token.clone();
            let file_name = chunk_file_info.name.clone();
            let chunk_hash = hashes[i as usize].clone();

            // Spawn a new asynchronous task for each range request
            let handle = task::spawn(async move {
                let _ = download_chunk_and_write_to_file(
                    &client,
                    &url,
                    auth_token,
                    &file_name,
                    start,
                    end,
                    &chunk_hash,
                    file_clone,
                )
                .await;
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete and collect the results
        for handle in handles {
            let res = handle.await?;
            info!(chunk_download_result = tracing::field::debug(&res));
        }

        Ok(())
    }
}

/// Make request to download a chunk and write it to the file in position
async fn download_chunk_and_write_to_file(
    http_client: &Client,
    query_endpoint: &str,
    auth_token: Option<String>,
    file_name: &str,
    start: u64,
    end: u64,
    chunk_hash: &str,
    file: Arc<Mutex<File>>,
) -> Result<(), anyhow::Error> {
    // Make the range request to download the chunk
    let data = request_chunk(
        http_client,
        query_endpoint,
        auth_token,
        file_name,
        start,
        end,
    )
    .await?;
    let downloaded_chunk_hash = hash_chunk(&data);

    // Verify the chunk by reading the chunk file and
    if &downloaded_chunk_hash != chunk_hash {
        tracing::warn!(query_endpoint, "Failed to validate a chunk from indexer");
        return Err(anyhow!("Terminate the download, blacklist the indexer"));
    }

    // Lock the file for writing
    let mut file = file.lock().await;

    // Seek to the start position of this chunk
    file.seek(SeekFrom::Start(start)).unwrap();

    // Write the chunk to the file
    file.write_all(&data).unwrap();

    Ok(())
}

/// Make range request for a file to the subfile server
async fn request_chunk(
    http_client: &Client,
    query_endpoint: &str,
    auth_token: Option<String>,
    file_name: &str,
    start: u64,
    end: u64,
) -> Result<Bytes, anyhow::Error> {
    // For example, to request the first 1024 bytes
    // The client should be smart enough to take care of proper chunking through subfile metadata
    let range = format!("bytes={}-{}", start, end);
    //TODO: implement payment flow
    // if auth_token.is_none() {
    //     tracing::error!(
    //         "No auth token provided; require receipt implementation"
    //     );
    //     Err(anyhow!("No auth token"))
    // };

    tracing::debug!(query_endpoint, range, "Make range request");
    let response = http_client
        .get(query_endpoint)
        .header("file_name", file_name)
        .header(CONTENT_RANGE, range)
        .header(
            AUTHORIZATION,
            auth_token.expect("No payment nor auth token"),
        )
        .send()
        .await?;

    // Check if the server supports range requests
    if response.status().is_success() && response.headers().contains_key(CONTENT_RANGE) {
        Ok(response.bytes().await?)
    } else {
        tracing::error!(
            status = tracing::field::debug(&response.status()),
            headers = tracing::field::debug(&response.headers()),
            chunk = tracing::field::debug(&response),
            "Server does not support range requests or the request failed"
        );
        Err(anyhow!("Range request failed"))
    }
}
