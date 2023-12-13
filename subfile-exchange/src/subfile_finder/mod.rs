use futures::{stream, StreamExt};

use std::collections::HashMap;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::errors::Error;

use crate::ipfs::IpfsClient;

use crate::subfile_reader::{fetch_subfile_from_ipfs, read_subfile};
use crate::subfile_server::util::Operator;

// Pair indexer operator address and indexer service endpoint (operator, indexer_url)
// persumeably this should not be handled by clients themselves
//TODO: smarter type for tracking available endpoints
pub type IndexerEndpoint = (String, String);
// Pair HashMap< ChunkFileIPFS, HashMap< IndexerEndpoint, Vec< MatchedSubfileIPFS > > >
pub type FileAvailbilityMap =
    Arc<Mutex<HashMap<String, Arc<Mutex<HashMap<IndexerEndpoint, Vec<String>>>>>>>;

pub struct SubfileFinder {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
}

impl SubfileFinder {
    pub fn new(ipfs_client: IpfsClient) -> Self {
        SubfileFinder {
            ipfs_client,
            http_client: reqwest::Client::new(),
        }
    }

    /// Endpoint must serve operator info and the requested file
    async fn subfile_availability(
        &self,
        subfile_hash: &str,
        url: &str,
    ) -> Result<IndexerEndpoint, Error> {
        let files = self.indexer_status(url).await?;
        let operator: String = self.indexer_operator(url).await?;

        if !files.contains(&subfile_hash.to_string()) {
            tracing::trace!(
                url,
                files = tracing::field::debug(&files),
                "IPFS hash not found in served subfile status"
            );
            return Err(Error::DataUnavilable(format!(
                "IPFS hash not found in files served at {}",
                url
            )));
        }

        Ok((operator, url.to_string()))
    }

    /// Check the availability of a subfile at various indexer endpoints
    /// Return a list of endpoints where the desired subfile is hosted
    pub async fn subfile_availabilities(
        &self,
        subfile_hash: &str,
        endpoint_checklist: &[String],
    ) -> Vec<IndexerEndpoint> {
        tracing::debug!(subfile_hash, "Checking availability");

        // Use a stream to process the endpoints in parallel
        let results = stream::iter(endpoint_checklist)
            .map(|url| self.subfile_availability(subfile_hash, url))
            .buffer_unordered(endpoint_checklist.len()) // Parallelize up to the number of endpoints
            .collect::<Vec<Result<IndexerEndpoint, Error>>>()
            .await;

        tracing::trace!(
            endpoints = tracing::field::debug(&results),
            "Endpoint availability result"
        );
        // Collect only the successful results
        results
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<IndexerEndpoint>>()
    }

    pub async fn file_discovery(
        &self,
        subfile_hash: &str,
        endpoint_checklist: &[String],
    ) -> Result<FileAvailbilityMap, Error> {
        let subfile = read_subfile(&self.ipfs_client, subfile_hash, PathBuf::new()).await?;
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

        match unavailble_files(&file_map).await {
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

    async fn indexer_operator(&self, url: &str) -> Result<String, Error> {
        let operator_url = format!("{}/operator", url);
        let operator_response = match self.http_client.get(&operator_url).send().await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Operator request failed for {}", operator_url);
                return Err(Error::Request(e));
            }
        };

        if !operator_response.status().is_success() {
            tracing::error!("Operator response error for {}", operator_url);
            return Err(Error::DataUnavilable(
                "Operator request failed.".to_string(),
            ));
        }

        match operator_response.json::<Operator>().await {
            Ok(operator) => Ok(operator.public_key),
            Err(e) => {
                tracing::error!("Operator response parse error for {}", operator_url);
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
                return Err(Error::Request(e));
            }
        };

        if !status_response.status().is_success() {
            let err_msg = format!("Status response errored: {}", status_url);
            return Err(Error::DataUnavilable(err_msg));
        }

        let files = match status_response.json::<Vec<String>>().await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Status response parse error for {}", status_url);
                return Err(Error::Request(e));
            }
        };

        Ok(files)
    }
}

/// Check if there is a key in target_hashes where the corresponding availability is empty
pub async fn unavailble_files(file_map: &FileAvailbilityMap) -> Vec<String> {
    let mut missing_file = vec![];
    let hashes = file_map.lock().await;
    for (key, inner_map_arc) in hashes.iter() {
        let inner_map = inner_map_arc.lock().await;
        if inner_map.is_empty() {
            missing_file.push(key.clone());
        }
    }
    missing_file
}
