use anyhow::anyhow;
use bytes::Bytes;
use futures::{stream, StreamExt};
use http::header::{AUTHORIZATION, CONTENT_RANGE};
use rand::seq::SliceRandom;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::DownloaderArgs;
use crate::file_hasher::verify_chunk;
use crate::ipfs::IpfsClient;
use crate::subfile::{FileMetaInfo, SubfileManifest};
use crate::subfile_reader::{fetch_chunk_file_from_ipfs, fetch_subfile_from_ipfs};
use crate::subfile_server::util::Operator;

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
    indexer_blocklist: Arc<StdMutex<HashSet<String>>>,
    // key is the chunk file identifier (IPFS hash) and value is a HashSet of downloaded chunk indices
    chunks_to_download: Arc<Mutex<HashMap<String, HashSet<u64>>>>,
    chunk_max_retry: u64,
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
            indexer_blocklist: Arc::new(StdMutex::new(HashSet::new())),
            chunks_to_download: Arc::new(Mutex::new(HashMap::new())),
            chunk_max_retry: args.max_retry,
        }
    }

    /// Check the availability of a subfile, ideally this should go through a gateway/DHT
    /// but for now we ping an indexer endpoint directly, which is what a gateway
    /// would do in behave of the downloader
    /// Return a list of endpoints where the desired subfile is hosted
    //TODO: update once there's a gateway with indexer selection providing endpoints
    pub async fn check_availability(&self) -> Result<Vec<IndexerEndpoint>, anyhow::Error> {
        tracing::info!("Checking availability");

        // Avoid blocked endpoints
        let blocklist = self
            .indexer_blocklist
            .lock()
            .map_err(|e| anyhow!("Cannot unwrap indexer_blocklist: {}", e.to_string()))?
            .clone();
        let filtered_endpoints = self
            .static_endpoints
            .iter()
            .filter(|url| !blocklist.contains(*url))
            .cloned()
            .collect::<Vec<_>>();
        // Use a stream to process the endpoints in parallel
        let results = stream::iter(&filtered_endpoints)
            .map(|url| self.check_endpoint_availability(url))
            .buffer_unordered(filtered_endpoints.len()) // Parallelize up to the number of endpoints
            .collect::<Vec<Result<IndexerEndpoint, anyhow::Error>>>()
            .await;

        tracing::debug!(
            endpoints = tracing::field::debug(&results),
            blocklist = tracing::field::debug(&self.indexer_blocklist),
            "Endpoint availability result"
        );
        // Collect only the successful results
        let endpoints = results
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        Ok(endpoints)
    }

    pub fn add_to_indexer_blocklist(&self, endpoint: String) {
        let mut blocklist = self.indexer_blocklist.lock().expect("Failed to lock mutex");
        blocklist.insert(endpoint);
    }

    /// Read manifest to prepare chunks download
    pub async fn chunks_to_download(&self) -> Result<SubfileManifest, anyhow::Error> {
        let subfile = fetch_subfile_from_ipfs(&self.ipfs_client, &self.ipfs_hash).await?;
        for chunk_file in &subfile.files {
            let mut chunks_to_download = self.chunks_to_download.lock().await;
            let chunks_set = chunks_to_download
                .entry(chunk_file.hash.clone())
                .or_insert_with(HashSet::new);
            let chunk_file =
                fetch_chunk_file_from_ipfs(&self.ipfs_client, &chunk_file.hash).await?;
            let chunk_size = chunk_file.chunk_size;
            for i in 0..(chunk_file.total_bytes / chunk_size + 1) {
                chunks_set.insert(i);
            }
        }
        Ok(subfile)
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

        //TODO: use chunks_to_download indices
        for i in 0..(chunk_file.total_bytes / chunk_size + 1) {
            tracing::trace!(i, "Download chunk index");
            let start = i * chunk_size;
            let end = u64::min(start + chunk_size, chunk_file.total_bytes) - 1;
            let file_clone = Arc::clone(&file);
            let mut rng = rand::thread_rng();
            let endpoint = if let Some((operator, url)) = query_endpoints.choose(&mut rng).cloned()
            {
                tracing::debug!(
                    operator,
                    url,
                    chunk = i,
                    chunk_file = chunk_file.file_name,
                    "Querying operator"
                );
                url
            } else {
                let err_msg = "Could not choose an operator to query, data unavailable";
                tracing::warn!(err_msg);
                return Err(anyhow!(err_msg));
            };
            let url = endpoint + "/subfiles/id/" + &self.ipfs_hash;
            let chunk_file_hash = chunk_file_info.hash.to_string();
            let client = self.http_client.clone();
            let auth_token = self.free_query_auth_token.clone();
            let file_name = chunk_file_info.name.clone();
            let chunk_hash = hashes[i as usize].clone();
            let block_list = self.indexer_blocklist.clone();
            let chunks_to_download = self.chunks_to_download.clone();
            let max_retry = self.chunk_max_retry;
            // Spawn a new asynchronous task for each range request
            let handle = tokio::spawn(async move {
                match download_chunk_and_write_to_file(
                    &client,
                    &url,
                    auth_token,
                    &file_name,
                    start,
                    end,
                    &chunk_hash,
                    file_clone,
                    max_retry,
                )
                .await
                {
                    Ok(r) => {
                        // Update downloaded status
                        chunks_to_download
                            .lock()
                            .await
                            .entry(chunk_file_hash)
                            .or_insert_with(HashSet::new)
                            .remove(&i);
                        Ok(r)
                    }
                    Err(e) => {
                        // If the download fails, add the URL to the indexer_blocklist
                        //TODO: with Error enum, add blocklist based on the error
                        block_list
                            .lock()
                            .expect("Cannot access blocklist")
                            .insert(url);
                        Err(e)
                    }
                }
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

    /// Endpoint must serve operator info and the requested file
    async fn check_endpoint_availability(
        &self,
        url: &str,
    ) -> Result<IndexerEndpoint, anyhow::Error> {
        let status_url = format!("{}/status", url);
        let status_response = match self.http_client.get(&status_url).send().await {
            Ok(response) => response,
            Err(_) => {
                tracing::error!("Status request failed for {}", status_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(anyhow!("Status request failed."));
            }
        };

        if !status_response.status().is_success() {
            tracing::error!("Status request failed for {}", status_url);
            self.add_to_indexer_blocklist(url.to_string());
            return Err(anyhow!("Status request failed."));
        }

        let files = match status_response.json::<Vec<String>>().await {
            Ok(files) => files,
            Err(_) => {
                tracing::error!("Status response parse error for {}", status_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(anyhow!("Status response parse error."));
            }
        };

        if !files.contains(&self.ipfs_hash) {
            tracing::error!("IPFS hash not found in files served at {}", status_url);
            self.add_to_indexer_blocklist(url.to_string());
            return Err(anyhow!(
                "IPFS hash not found in files served at {}",
                status_url
            ));
        }

        let operator_url = format!("{}/operator", url);
        let operator_response = match self.http_client.get(&operator_url).send().await {
            Ok(response) => response,
            Err(_) => {
                tracing::error!("Operator request failed for {}", operator_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(anyhow!("Operator request failed."));
            }
        };

        if !operator_response.status().is_success() {
            tracing::error!("Operator request failed for {}", operator_url);
            self.add_to_indexer_blocklist(url.to_string());
            return Err(anyhow!("Operator request failed."));
        }

        match operator_response.json::<Operator>().await {
            Ok(operator) => Ok((operator.public_key, url.to_string())),
            Err(_) => {
                tracing::error!("Operator response parse error for {}", operator_url);
                self.add_to_indexer_blocklist(url.to_string());
                Err(anyhow!("Operator response parse error."))
            }
        }
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
    max_retry: u64,
) -> Result<Arc<Mutex<File>>, anyhow::Error> {
    let mut attempts = 0;

    loop {
        // Make the range request to download the chunk
        match request_chunk(
            http_client,
            query_endpoint,
            auth_token.clone(),
            file_name,
            start,
            end,
        )
        .await
        {
            Ok(data) => {
                if verify_chunk(&data, chunk_hash) {
                    // Lock the file for writing
                    let mut file_lock = file.lock().await;
                    file_lock.seek(SeekFrom::Start(start))?;
                    file_lock.write_all(&data)?;
                    drop(file_lock);
                    return Ok(file); // Successfully written the chunk, exit loop
                } else {
                    tracing::warn!(query_endpoint, "Failed to validate received chunk");
                }
            }
            Err(e) => tracing::error!("Chunk download error: {:?}", e),
        }

        attempts += 1;
        if attempts >= max_retry {
            return Err(anyhow!("Max retry attempts reached for chunk download"));
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
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
