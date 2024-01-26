use alloy_primitives::{Address, U256};
use bytes::Bytes;

use ethers::providers::{Http, Middleware, Provider};
use rand::seq::SliceRandom;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_RANGE},
    Client,
};
use secp256k1::SecretKey;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::DownloaderArgs;
use crate::discover::{Finder, IndexerEndpoint};
use crate::errors::Error;
use crate::manifest::{
    file_hasher::verify_chunk, ipfs::IpfsClient, manifest_fetcher::read_bundle, Bundle,
    FileManifestMeta,
};

use crate::util::build_wallet;

use self::signer::{ReceiptSigner, TapReceipt};

pub mod signer;

pub struct Downloader {
    http_client: reqwest::Client,
    ipfs_hash: String,
    bundle: Bundle,
    _gateway_url: Option<String>,
    static_endpoints: Vec<String>,
    output_dir: String,
    free_query_auth_token: Option<String>,
    indexer_urls: Arc<StdMutex<Vec<IndexerEndpoint>>>,
    indexer_blocklist: Arc<StdMutex<HashSet<String>>>,
    // key is the file manifest identifier (IPFS hash) and value is a HashSet of downloaded chunk indices
    target_chunks: Arc<StdMutex<HashMap<String, HashSet<u64>>>>,
    chunk_max_retry: u64,
    bundle_finder: Finder,
    #[allow(dead_code)]
    receipt_signer: ReceiptSigner,
}

impl Downloader {
    pub async fn new(ipfs_client: IpfsClient, args: DownloaderArgs) -> Self {
        let bundle = read_bundle(
            &ipfs_client,
            &args.ipfs_hash,
            args.output_dir.clone().into(),
        )
        .await
        .expect("Read bundle");

        let wallet = build_wallet(&args.mnemonic).expect("Mnemonic build wallet");
        let signing_key = wallet.signer().to_bytes();
        let secp256k1_private_key =
            SecretKey::from_slice(&signing_key).expect("Private key from wallet");
        let provider = Provider::<Http>::try_from(&args.provider).expect("Connect to the provider");
        //TODO: migrate ethers type to alloy
        let chain_id = U256::from(
            provider
                .get_chainid()
                .await
                .expect("Get chain id from provider")
                .as_u128(),
        );
        let receipt_signer = ReceiptSigner::new(
            secp256k1_private_key,
            chain_id,
            Address::from_str(&args.verifier).expect("Parse verifier"),
        )
        .await;

        Downloader {
            http_client: reqwest::Client::new(),
            ipfs_hash: args.ipfs_hash,
            bundle,
            _gateway_url: args.gateway_url,
            static_endpoints: args.indexer_endpoints,
            output_dir: args.output_dir,
            free_query_auth_token: args.free_query_auth_token,
            indexer_urls: Arc::new(StdMutex::new(Vec::new())),
            indexer_blocklist: Arc::new(StdMutex::new(HashSet::new())),
            target_chunks: Arc::new(StdMutex::new(HashMap::new())),
            chunk_max_retry: args.max_retry,
            bundle_finder: Finder::new(ipfs_client),
            receipt_signer,
        }
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
    pub fn target_chunks(&self, bundle: &Bundle) {
        for file_manifest_meta in &bundle.file_manifests {
            let mut target_chunks = self.target_chunks.lock().unwrap();
            let chunks_set = target_chunks
                .entry(file_manifest_meta.meta_info.hash.clone())
                .or_default();
            let chunk_size = file_manifest_meta.file_manifest.chunk_size;
            for i in 0..(file_manifest_meta.file_manifest.total_bytes / chunk_size + 1) {
                chunks_set.insert(i);
            }
        }
    }

    /// Read bundle manifiest and download the individual file manifests
    //TODO: update once there is payment
    pub async fn download_bundle(&self) -> Result<(), Error> {
        self.target_chunks(&self.bundle);
        tracing::info!(
            chunks = tracing::field::debug(self.target_chunks.clone()),
            "File manifests download starting"
        );

        // check bundle availability from gateway/indexer_endpoints
        let _ = self.availbility_check().await;

        // Loop through file manifests for downloading
        let mut incomplete_files = vec![];
        for file_manifest in &self.bundle.file_manifests {
            if let Err(e) = self.download_file_manifest(file_manifest.clone()).await {
                incomplete_files.push(e);
            }
        }

        //TODO: retry for failed bundles
        if !incomplete_files.is_empty() {
            let msg = format!(
                "File manifests download incomplete: {:#?}",
                tracing::field::debug(&incomplete_files),
            );
            tracing::warn!(msg);
            return Err(Error::DataUnavilable(msg));
        } else {
            tracing::info!("File manifests download completed");
        }
        Ok(())
    }

    /// Get the remaining chunks to download for a file
    pub fn remaining_chunks(&self, file_manifest_hash: &String) -> Vec<u64> {
        self.target_chunks
            .lock()
            .unwrap()
            .get(file_manifest_hash)
            .map(|chunks| chunks.clone().into_iter().collect())
            .unwrap_or_default()
    }

    /// Download a file by reading its chunk manifest
    //TODO: update once there is payment
    pub async fn download_file_manifest(&self, meta: FileManifestMeta) -> Result<(), Error> {
        tracing::debug!(
            file_spec = tracing::field::debug(&meta),
            "Download file manifest"
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
                let file_manifest_hash = meta.meta_info.hash.clone();
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
                                    .entry(file_manifest_hash)
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
                                    "File manifest download incomplete"
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

    /// Generate a request to download a chunk
    fn download_range_request(
        &self,
        meta: &FileManifestMeta,
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
                file_manifest = meta.meta_info.hash,
                "Querying operator"
            );
            url.clone()
        } else {
            let err_msg = "No operator serving the file, data unavailable".to_string();
            tracing::warn!(err_msg);
            return Err(Error::DataUnavilable(err_msg.to_string()));
        };
        //TODO: do no add ipfs_hash here, construct query_endpoint after updating route 'bundles/id/:id'
        let query_endpoint = url + "/bundles/id/" + &self.ipfs_hash;
        let file_hash = meta.meta_info.hash.clone();
        let start = i * meta.file_manifest.chunk_size;
        let end = u64::min(
            start + meta.file_manifest.chunk_size,
            meta.file_manifest.total_bytes,
        ) - 1;
        let chunk_hash = meta.file_manifest.chunk_hashes[i as usize].clone();

        Ok(DownloadRangeRequest {
            query_endpoint,
            file_hash,
            start,
            end,
            chunk_hash,
            file,
            max_retry: self.chunk_max_retry,
            auth_token: self.free_query_auth_token.clone(),
            _receipt: None,
        })
    }

    async fn availbility_check(&self) -> Result<(), Error> {
        let blocklist = self.indexer_blocklist.lock().unwrap().clone();
        let endpoints = &self
            .static_endpoints
            .iter()
            .filter(|url| !blocklist.contains(*url))
            .cloned()
            .collect::<Vec<_>>();
        self.update_indexer_urls(
            &self
                .bundle_finder
                .bundle_availabilities(&self.ipfs_hash, endpoints)
                .await,
        );
        let indexer_endpoints = self.indexer_urls.lock().unwrap().clone();
        if indexer_endpoints.is_empty() {
            tracing::warn!(
                bundle_hash = &self.ipfs_hash,
                "No endpoint satisfy the bundle requested, sieve through available bundles for individual files"
            );

            // check files availability from gateway/indexer_endpoints
            match self
                .bundle_finder
                .file_discovery(&self.ipfs_hash, endpoints)
                .await
            {
                Ok(map) => {
                    let msg = format!(
                        "Files available on these available bundles: {:#?}",
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
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DownloadRangeRequest {
    query_endpoint: String,
    auth_token: Option<String>,
    _receipt: Option<TapReceipt>,
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

/// Make range request for a file to the bundle server
async fn request_chunk(
    http_client: &Client,
    query_endpoint: &str,
    auth_token: Option<String>,
    file_hash: &str,
    start: u64,
    end: u64,
) -> Result<Bytes, Error> {
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

/// extract base indexer_url from `indexer_url/bundles/id/bundle_id`
fn extract_base_url(query_endpoint: &str) -> Option<&str> {
    if let Some(index) = query_endpoint.find("/bundles/id/") {
        Some(&query_endpoint[..index])
    } else {
        None
    }
}