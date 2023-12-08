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
use crate::errors::Error;
use crate::file_hasher::verify_chunk;
use crate::ipfs::IpfsClient;
use crate::subfile::{ChunkFileMeta, Subfile};
use crate::subfile_reader::{fetch_subfile_from_ipfs, read_subfile};
use crate::subfile_server::util::Operator;

// Pair indexer operator address and indexer service endpoint (operator, indexer_url)
// persumeably this should not be handled by clients themselves
//TODO: smarter type for tracking available endpoints
pub type IndexerEndpoint = (String, String);
pub type FileAvailbilityMap =
    Arc<Mutex<HashMap<String, Arc<Mutex<HashMap<IndexerEndpoint, Vec<String>>>>>>>;

pub struct SubfileDownloader {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
    ipfs_hash: String,
    _gateway_url: Option<String>,
    static_endpoints: Vec<String>,
    output_dir: String,
    free_query_auth_token: Option<String>,
    indexer_urls: Arc<StdMutex<Vec<IndexerEndpoint>>>,
    indexer_blocklist: Arc<StdMutex<HashSet<String>>>,
    // key is the chunk file identifier (IPFS hash) and value is a HashSet of downloaded chunk indices
    target_chunks: Arc<StdMutex<HashMap<String, HashSet<u64>>>>,
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
            indexer_urls: Arc::new(StdMutex::new(Vec::new())),
            indexer_blocklist: Arc::new(StdMutex::new(HashSet::new())),
            target_chunks: Arc::new(StdMutex::new(HashMap::new())),
            chunk_max_retry: args.max_retry,
        }
    }

    /// Check the availability of a subfile, ideally this should go through a gateway/DHT
    /// but for now we ping an indexer endpoint directly, which is what a gateway
    /// would do in behave of the downloader
    /// Return a list of endpoints where the desired subfile is hosted
    //TODO: update once there's a gateway with indexer selection providing endpoints
    //TODO: Use eventuals for continuous pings
    //TODO: Availability by file hash
    pub async fn check_availability(&self, endpoint_checklist: &[String]) -> Result<(), Error> {
        tracing::debug!(subfile_hash = &self.ipfs_hash, "Checking availability");
        // Avoid blocked endpoints
        let blocklist = self.indexer_blocklist.lock().unwrap().clone();
        let filtered_endpoints = endpoint_checklist
            .iter()
            .filter(|url| !blocklist.contains(*url))
            .cloned()
            .collect::<Vec<_>>();
        // Use a stream to process the endpoints in parallel
        let results = stream::iter(&filtered_endpoints)
            .map(|url| self.subfile_availability(url))
            .buffer_unordered(filtered_endpoints.len()) // Parallelize up to the number of endpoints
            .collect::<Vec<Result<IndexerEndpoint, Error>>>()
            .await;

        tracing::trace!(
            endpoints = tracing::field::debug(&results),
            blocklist = tracing::field::debug(&self.indexer_blocklist),
            "Endpoint availability result"
        );
        // Collect only the successful results
        let endpoints = results
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<IndexerEndpoint>>();
        self.update_indexer_urls(&endpoints);

        if endpoints.is_empty() {
            Err(Error::DataUnavilable(String::from(
                "No eligible endpoints serving the subfile",
            )))
        } else {
            Ok(())
        }
    }

    pub async fn file_discovery(
        &self,
        endpoint_checklist: &[String],
    ) -> Result<FileAvailbilityMap, Error> {
        let subfile = read_subfile(
            &self.ipfs_client,
            &self.ipfs_hash,
            self.output_dir.clone().into(),
        )
        .await?;
        // To fill in availability for each file, get a vector of (IndexerEndpoint, SubfileIPFS) that serves the file
        let target_hashes: FileAvailbilityMap = Arc::new(Mutex::new(
            subfile
                .chunk_files
                .iter()
                .map(|chunk_file| {
                    (
                        chunk_file.meta_info.hash.clone(),
                        Arc::new(Mutex::new(HashMap::new())),
                    )
                })
                .collect(),
        ));

        for url in endpoint_checklist {
            if let Err(_e) = self.file_availability(url, target_hashes.clone()).await {
                tracing::debug!("Failed to get file availability: {:#?}", url);
            };
        }

        tracing::info!("Discovered file availability map: {:#?}", target_hashes);
        Ok(target_hashes)
    }

    /// Gather file availability
    pub async fn file_availability(
        &self,
        url: &str,
        file_map: FileAvailbilityMap,
    ) -> Result<(), Error> {
        let operator = self.indexer_operator(url).await?;
        let indexer_endpoint = (operator, url.to_string());
        let subfiles = self.indexer_status(url).await?;

        // Map of indexer_endpoints to served Subfiles
        // For each endpoint, populate indexer_map with the available files
        for subfile in subfiles {
            let file_hashes: Vec<String> = fetch_subfile_from_ipfs(&self.ipfs_client, &subfile)
                .await?
                .files
                .iter()
                .map(|file| file.hash.clone())
                .collect();
            let file_map_lock = file_map.lock().await;
            for (target_file, availability_map) in file_map_lock.iter() {
                // Record serving indexer and subfile for each target file
                if file_hashes.contains(target_file) {
                    availability_map
                        .lock()
                        .await
                        .entry(indexer_endpoint.clone())
                        .and_modify(|e| e.push(subfile.clone()))
                        .or_insert(vec![subfile.clone()]);
                }
            }
        }

        match contains_key_with_empty_map(&file_map).await {
            files if !files.is_empty() => {
                return Err(Error::DataUnavilable(format!(
                    "File availability incomplete, missing files: {:#?}",
                    files
                )));
            }
            _ => {}
        }

        // Return the map of file hash to serving indexer
        Ok(())
    }

    pub fn update_indexer_urls(&self, endpoints: &[IndexerEndpoint]) {
        self.indexer_urls.lock().unwrap().clear();
        self.indexer_urls
            .lock()
            .unwrap()
            .extend(endpoints.to_owned());
    }

    pub fn add_to_indexer_blocklist(&self, endpoint: String) {
        let mut blocklist = self.indexer_blocklist.lock().expect("Failed to lock mutex");
        blocklist.insert(endpoint);
    }

    /// Read manifest to prepare chunks download
    pub fn target_chunks(&self, subfile: &Subfile) {
        for chunk_file_meta in &subfile.chunk_files {
            let mut target_chunks = self.target_chunks.lock().unwrap();
            let chunks_set = target_chunks
                .entry(chunk_file_meta.meta_info.hash.clone())
                .or_default();
            let chunk_size = chunk_file_meta.chunk_file.chunk_size;
            for i in 0..(chunk_file_meta.chunk_file.total_bytes / chunk_size + 1) {
                chunks_set.insert(i);
            }
        }
    }

    /// Read subfile manifiest and download the individual chunk files
    //TODO: update once there is payment
    pub async fn download_subfile(&self) -> Result<(), Error> {
        let subfile = read_subfile(
            &self.ipfs_client,
            &self.ipfs_hash,
            self.output_dir.clone().into(),
        )
        .await?;
        self.target_chunks(&subfile);
        tracing::info!(
            chunks = tracing::field::debug(self.target_chunks.clone()),
            "Chunk files download starting"
        );

        // check subfile availability from gateway/indexer_endpoints
        if let Err(e) = self.check_availability(&self.static_endpoints).await {
            tracing::warn!(
                query_endpoints = tracing::field::debug(&self.indexer_urls.lock().unwrap()),
                err = tracing::field::debug(&e),
                "No endpoint satisfy the subfile requested, sieve through available subfiles for individual files"
            );

            // check files availability from gateway/indexer_endpoints
            match self.file_discovery(&self.static_endpoints).await {
                Ok(map) => {
                    let msg = format!(
                        "Files available on other available subfiles: {:#?}",
                        tracing::field::debug(&map.lock().await),
                    );
                    return Err(Error::DataUnavilable(msg));
                }
                Err(e) => {
                    let msg = format!(
                        "Cannot match the files: {:?}, {:?}",
                        tracing::field::debug(&self.indexer_urls.lock().unwrap()),
                        tracing::field::debug(&e),
                    );
                    tracing::error!(msg);
                    return Err(Error::DataUnavilable(msg));
                }
            }
        };

        // Loop through chunk files for downloading
        let mut incomplete_files = vec![];
        for chunk_file in &subfile.chunk_files {
            if let Err(e) = self.download_chunk_file(chunk_file.clone()).await {
                incomplete_files.push(e);
            }
        }

        //TODO: retry for failed subfiles
        if !incomplete_files.is_empty() {
            let msg = format!(
                "Chunk files download incomplete: {:#?}",
                tracing::field::debug(&incomplete_files),
            );
            tracing::warn!(msg);
            return Err(Error::DataUnavilable(msg));
        } else {
            tracing::info!("Chunk files download completed");
        }
        Ok(())
    }

    /// Get the remaining chunks to download for a file
    pub fn remaining_chunks(&self, chunk_file_hash: &String) -> Vec<u64> {
        self.target_chunks
            .lock()
            .unwrap()
            .get(chunk_file_hash)
            .map(|chunks| chunks.clone().into_iter().collect())
            .unwrap_or_default()
    }

    /// Download a file by reading its chunk manifest
    //TODO: update once there is payment
    pub async fn download_chunk_file(&self, meta: ChunkFileMeta) -> Result<(), Error> {
        tracing::debug!(
            file_spec = tracing::field::debug(&meta),
            "Download chunk file"
        );

        // Open the output file
        let file = File::create(Path::new(
            &(self.output_dir.clone() + "/" + &meta.meta_info.name),
        ))
        .unwrap();
        let file = Arc::new(Mutex::new(file));

        while !self.remaining_chunks(&meta.meta_info.hash).is_empty() {
            // Wait for all chunk tasks to complete and collect the results
            let mut handles = Vec::new();
            for i in self.remaining_chunks(&meta.meta_info.hash) {
                let chunk_file_hash = meta.meta_info.hash.clone();
                let client = self.http_client.clone();
                //TODO: can utilize operator address for on-chain checks
                let request = self.download_range_request(&meta, i, file.clone())?;
                let block_list = self.indexer_blocklist.clone();
                let target_chunks = self.target_chunks.clone();
                let url = request.query_endpoint.clone();
                // Spawn a new asynchronous task for each range request
                let handle: tokio::task::JoinHandle<Result<Arc<Mutex<File>>, Error>> =
                    tokio::spawn(async move {
                        match download_chunk_and_write_to_file(&client, request).await {
                            Ok(r) => {
                                // Update downloaded status
                                target_chunks
                                    .lock()
                                    .unwrap()
                                    .entry(chunk_file_hash)
                                    .or_default()
                                    .remove(&i);
                                tracing::trace!(i, "Chunk downloaded");
                                Ok(r)
                            }
                            Err(e) => {
                                // If the download fails, add the URL to the indexer_blocklist
                                let url = match extract_base_url(&url) {
                                    Some(url) => url.to_string(),
                                    None => url,
                                };
                                tracing::warn!(
                                    err = e.to_string(),
                                    "Chunk file download incomplete"
                                );
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

            for handle in handles {
                let _ = handle
                    .await
                    .map_err(|e| Error::DataUnavilable(e.to_string()))?;
            }
        }

        tracing::info!(
            chunks = tracing::field::debug(self.target_chunks.clone()),
            file_info = tracing::field::debug(&meta.meta_info),
            "File finished"
        );
        Ok(())
    }

    async fn indexer_operator(&self, url: &str) -> Result<String, Error> {
        let operator_url = format!("{}/operator", url);
        let operator_response = match self.http_client.get(&operator_url).send().await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Operator request failed for {}", operator_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(Error::Request(e));
            }
        };

        if !operator_response.status().is_success() {
            tracing::error!("Operator request failed for {}", operator_url);
            self.add_to_indexer_blocklist(url.to_string());
            return Err(Error::DataUnavilable(
                "Operator request failed.".to_string(),
            ));
        }

        match operator_response.json::<Operator>().await {
            Ok(operator) => Ok(operator.public_key),
            Err(e) => {
                tracing::error!("Operator response parse error for {}", operator_url);
                self.add_to_indexer_blocklist(url.to_string());
                Err(Error::Request(e))
            }
        }
    }

    async fn indexer_status(&self, url: &str) -> Result<Vec<String>, Error> {
        let status_url = format!("{}/status", url);
        let status_response = match self.http_client.get(&status_url).send().await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Status request failed for {}", status_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(Error::Request(e));
            }
        };

        if !status_response.status().is_success() {
            let err_msg = format!("Status request unsuccessful for {}", status_url);
            self.add_to_indexer_blocklist(url.to_string());
            return Err(Error::DataUnavilable(err_msg));
        }

        let files = match status_response.json::<Vec<String>>().await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Status response parse error for {}", status_url);
                self.add_to_indexer_blocklist(url.to_string());
                return Err(Error::Request(e));
            }
        };

        Ok(files)
    }

    /// Endpoint must serve operator info and the requested file
    async fn subfile_availability(&self, url: &str) -> Result<IndexerEndpoint, Error> {
        let files = self.indexer_status(url).await?;
        let operator: String = self.indexer_operator(url).await?;

        if !files.contains(&self.ipfs_hash) {
            tracing::trace!(
                url,
                files = tracing::field::debug(&files),
                "IPFS hash not found in served subfile status"
            );
            self.add_to_indexer_blocklist(url.to_string());
            return Err(Error::DataUnavilable(format!(
                "IPFS hash not found in files served at {}",
                url
            )));
        }

        Ok((operator, url.to_string()))
    }

    /// Generate a request to download a chunk
    fn download_range_request(
        &self,
        meta: &ChunkFileMeta,
        i: u64,
        file: Arc<Mutex<File>>,
    ) -> Result<DownloadRangeRequest, Error> {
        let mut rng = rand::thread_rng();
        let query_endpoints = &self.indexer_urls.lock().unwrap();
        let url = if let Some((operator, url)) = query_endpoints.choose(&mut rng).cloned() {
            tracing::debug!(
                operator,
                url,
                chunk = i,
                chunk_file = meta.meta_info.hash,
                "Querying operator"
            );
            url.clone()
        } else {
            let err_msg = "No operator serving the file, data unavailable".to_string();
            tracing::warn!(err_msg);
            return Err(Error::DataUnavilable(err_msg.to_string()));
        };
        //TODO: do no add ipfs_hash here, construct query_endpoint after updating route 'subfiles/id/:id'
        let query_endpoint = url + "/subfiles/id/" + &self.ipfs_hash;
        let file_hash = meta.meta_info.hash.clone();
        let start = i * meta.chunk_file.chunk_size;
        let end = u64::min(
            start + meta.chunk_file.chunk_size,
            meta.chunk_file.total_bytes,
        ) - 1;
        let chunk_hash = meta.chunk_file.chunk_hashes[i as usize].clone();
        Ok(DownloadRangeRequest {
            query_endpoint,
            file_hash,
            start,
            end,
            chunk_hash,
            file,
            max_retry: self.chunk_max_retry,
            auth_token: self.free_query_auth_token.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DownloadRangeRequest {
    query_endpoint: String,
    auth_token: Option<String>,
    file_hash: String,
    start: u64,
    end: u64,
    chunk_hash: String,
    file: Arc<Mutex<File>>,
    max_retry: u64,
}

/// Make request to download a chunk and write it to the file in position
async fn download_chunk_and_write_to_file(
    http_client: &Client,
    request: DownloadRangeRequest,
) -> Result<Arc<Mutex<File>>, Error> {
    let mut attempts = 0;

    tracing::debug!(
        request = tracing::field::debug(&request),
        "Making a range request"
    );
    loop {
        // Make the range request to download the chunk
        match request_chunk(
            http_client,
            &request.query_endpoint,
            request.auth_token.clone(),
            &request.file_hash,
            request.start,
            request.end,
        )
        .await
        {
            Ok(data) => {
                if verify_chunk(&data, &request.chunk_hash) {
                    // Lock the file for writing
                    let mut file_lock = request.file.lock().await;
                    file_lock
                        .seek(SeekFrom::Start(request.start))
                        .map_err(Error::FileIOError)?;
                    file_lock.write_all(&data).map_err(Error::FileIOError)?;
                    drop(file_lock);
                    return Ok(request.file); // Successfully written the chunk, exit loop
                } else {
                    // Immediately return and blacklist the indexer when a chunk received is invalid
                    let msg = format!(
                        "Failed to validate received chunk: {}",
                        &request.query_endpoint
                    );
                    tracing::warn!(msg);
                    return Err(Error::ChunkInvalid(msg));
                }
            }
            Err(e) => tracing::error!("Chunk download error: {:?}", e),
        }

        attempts += 1;
        if attempts >= request.max_retry {
            return Err(Error::DataUnavilable(
                "Max retry attempts reached for chunk download".to_string(),
            ));
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Make range request for a file to the subfile server
async fn request_chunk(
    http_client: &Client,
    query_endpoint: &str,
    auth_token: Option<String>,
    file_hash: &str,
    start: u64,
    end: u64,
) -> Result<Bytes, Error> {
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
        .header("file_hash", file_hash)
        .header(CONTENT_RANGE, range)
        .header(
            AUTHORIZATION,
            auth_token.expect("No payment nor auth token"),
        )
        .send()
        .await
        .map_err(Error::Request)?;

    // Check if the server supports range requests
    if response.status().is_success() && response.headers().contains_key(CONTENT_RANGE) {
        Ok(response.bytes().await.map_err(Error::Request)?)
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
        Err(Error::InvalidRange(err_msg))
    }
}

/// extract base indexer_url from `indexer_url/subfiles/id/subfile_id`
fn extract_base_url(query_endpoint: &str) -> Option<&str> {
    if let Some(index) = query_endpoint.find("/subfiles/id/") {
        Some(&query_endpoint[..index])
    } else {
        None
    }
}

/// Check if there is a key in target_hashes where the corresponding availability is empty
async fn contains_key_with_empty_map(file_map: &FileAvailbilityMap) -> Vec<String> {
    let mut missing_file = vec![];
    let hashes = file_map.lock().await;
    for (key, inner_map_arc) in hashes.iter() {
        let inner_map = inner_map_arc.lock().await; // Lock the Mutex to access the inner map
        if inner_map.is_empty() {
            missing_file.push(key.clone());
        }
    }
    missing_file
}
