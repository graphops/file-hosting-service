// #![deny(warnings)]
// use hyper;

// use std::env;
// use std::io::{self, Write};
// use hyper::Client;
// use hyper::header::{Connection, Range, ByteRangeSpec};

use anyhow::anyhow;
use bytes::Bytes;
use http::header::{AUTHORIZATION, CONTENT_RANGE};
use rand::seq::SliceRandom;
use reqwest::Client;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

use crate::config::DownloaderArgs;
use crate::file_hasher::verify_chunk;
use crate::ipfs::IpfsClient;
use crate::publisher::FileMetaInfo;
use crate::subfile_reader::{fetch_chunk_file_from_ipfs, fetch_subfile_from_ipfs};
use crate::types::Operator;

// Pair indexer operator address and indexer service endpoint
// persumeably this should not be handled by clients themselves
//TODO: smarter type for tracking available endpoints
pub type IndexerEndpoint = (String, String);

pub struct SubfileDownloader {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
    ipfs_hash: String,
    _gateway_url: Option<String>,
    static_endpoints: Vec<String>,
    output_dir: String,
    free_query_auth_token: Option<String>,
    // Other fields as needed
}

impl SubfileDownloader {
    pub fn new(ipfs_client: IpfsClient, args: DownloaderArgs) -> Self {
        SubfileDownloader {
            ipfs_client,
            //TODO: consider a more advanced config such as if a proxy should be used for the client
            http_client: reqwest::Client::new(),
            ipfs_hash: args.ipfs_hash,
            _gateway_url: args.gateway_url,
            //TODO: Check for healthy indexers in args.indexer_endpoints
            static_endpoints: args.indexer_endpoints,
            output_dir: args.output_dir,
            free_query_auth_token: args.free_query_auth_token,
        }
    }

    /// Check the availability of a subfile, ideally this should go through a gateway/DHT
    /// but for now we ping an indexer endpoint directly, which is what a gateway
    /// would do in behave of the downloader
    /// Return a list of endpoints where the desired subfile is hosted
    //TODO: update once there's a gateway
    pub async fn check_availability(&self) -> Result<Vec<IndexerEndpoint>, anyhow::Error> {
        tracing::info!("Check availability");

        let mut endpoints = vec![];
        // Loop through indexer endpoints and query data availability for the desired subfile
        //TODO: parallelize
        for url in &self.static_endpoints {
            // Ping availability endpoint
            let status_url = url.to_owned() + "/status";
            let status_response = self.http_client.get(&status_url).send().await?;

            if status_response.status().is_success() {
                tracing::info!(status_url = &status_url, "Successful status response");
                if let Ok(files) = status_response.json::<Vec<String>>().await {
                    if files.contains(&self.ipfs_hash) {
                        let operator_url = url.to_owned() + "/operator";
                        let operator_response = self.http_client.get(&operator_url).send().await?;
                        if operator_response.status().is_success() {
                            tracing::info!(
                                operator_url = &operator_url,
                                "Successful operator response"
                            );
                            //TODO: add indexer pricing to do basic byte prices matching
                            if let Ok(operator) = operator_response.json::<Operator>().await {
                                endpoints.push((operator.public_key, url.clone()));
                            } else {
                                tracing::error!("Operator response parse error.");
                                return Err(anyhow!("Operator response parse error."));
                            }
                        } else {
                            tracing::error!("Operator request failed.");
                            return Err(anyhow!("Operator request failed."));
                        }
                    }
                } else {
                    tracing::error!("Status request parse error.");
                    return Err(anyhow!("Status request parse error."));
                }
            } else {
                tracing::error!("Status request failed.");
                return Err(anyhow!("Status request failed."));
            }
        }
        Ok(endpoints)
    }

    /// Read subfile manifiest and download the individual chunk files
    //TODO: update once there is payment
    pub async fn download_subfile(&self) -> Result<(), anyhow::Error> {
        // Read subfile from ipfs
        let subfile = fetch_subfile_from_ipfs(&self.ipfs_client, &self.ipfs_hash).await?;

        // Loop through chunk files for downloading
        let mut incomplete_files = vec![];
        for chunk_file in &subfile.files {
            match self.download_chunk_file(chunk_file).await {
                Ok(_) => {}
                Err(e) => incomplete_files.push(e),
            }
        }

        //TODO: retry for failed subfiles
        if !incomplete_files.is_empty() {
            let msg = format!(
                "Chunk files download incomplete: {:#?}",
                tracing::field::debug(&incomplete_files),
            );
            tracing::warn!(msg);
            return Err(anyhow!(msg));
        } else {
            tracing::info!("Chunk files download completed");
        }
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
        //TODO: make gateway ISA
        let query_endpoints = self.check_availability().await?;
        tracing::debug!(
            query_endpoints = tracing::field::debug(&query_endpoints),
            "Basic matching with query availability"
        );

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
            let mut rng = rand::thread_rng();
            let url = if let Some((operator, url)) = query_endpoints.choose(&mut rng).cloned() {
                tracing::debug!(
                    operator,
                    url,
                    chunk = i,
                    chunk_file = chunk_file_info.name,
                    "Querying operator"
                );
                url + "/subfiles/id/" + &self.ipfs_hash
            } else {
                let err_msg = "Could not choose an operator to query, data unavailable";
                tracing::warn!(err_msg);
                return Err(anyhow!(err_msg));
            };
            let client = self.http_client.clone();
            let auth_token = self.free_query_auth_token.clone();
            let file_name = chunk_file_info.name.clone();
            let chunk_hash = hashes[i as usize].clone();

            // Spawn a new asynchronous task for each range request
            let handle = task::spawn(async move {
                download_chunk_and_write_to_file(
                    &client,
                    &url,
                    auth_token,
                    &file_name,
                    start,
                    end,
                    &chunk_hash,
                    file_clone,
                )
                .await
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete and collect the results
        let mut failed = vec![];
        for handle in handles {
            match handle.await? {
                Ok(file) => {
                    let metadata = file.lock().await.metadata()?;

                    let modified = if let Ok(time) = metadata.modified() {
                        format!("Modified: {:#?}", time)
                    } else {
                        "Not modified".to_string()
                    };

                    tracing::info!(
                        metadata = tracing::field::debug(metadata),
                        modification = modified,
                        "Chunk file information"
                    );
                }
                Err(e) => {
                    tracing::warn!(err = e.to_string(), "Chunk file download incomplete");
                    failed.push(e.to_string());
                }
            }
        }

        //TODO: track incompletness
        if !failed.is_empty() {
            return Err(anyhow!("Failed chunks: {:#?}", failed));
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
) -> Result<Arc<Mutex<File>>, anyhow::Error> {
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

    // Verify the chunk by reading the chunk file and
    if !verify_chunk(&data, chunk_hash) {
        tracing::warn!(query_endpoint, "Failed to validate a chunk from indexer");
        return Err(anyhow!("Terminate the download, blacklist the indexer"));
    }

    // Lock the file for writing
    let mut file_lock = file.lock().await;

    // Seek to the start position of this chunk
    file_lock.seek(SeekFrom::Start(start)).unwrap();

    // Write the chunk to the file
    file_lock.write_all(&data).unwrap();

    drop(file_lock);

    Ok(file)
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
        let err_msg = format!(
            "Server does not support range requests or the request failed: {:#?}",
            tracing::field::debug(&response.status()),
        );
        tracing::error!(
            status = tracing::field::debug(&response.status()),
            headers = tracing::field::debug(&response.headers()),
            chunk = tracing::field::debug(&response),
            "Server does not support range requests or the request failed"
        );
        Err(anyhow!("Range request failed: {}", err_msg))
    }
}
