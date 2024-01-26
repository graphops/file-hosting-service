use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::errors::Error;

use crate::manifest::{
    ipfs::IpfsClient,
    manifest_fetcher::{fetch_bundle_from_ipfs, read_bundle},
};

// Pair indexer operator address and indexer service endpoint (operator, indexer_url)
// persumeably this should not be handled by clients themselves
//TODO: smarter type for tracking available endpoints
pub type IndexerEndpoint = (String, String);
// Pair HashMap< FileManifestIPFS, HashMap< IndexerEndpoint, Vec< MatchedManifestIPFS > > >
pub type FileAvailbilityMap =
    Arc<Mutex<HashMap<String, Arc<Mutex<HashMap<IndexerEndpoint, Vec<String>>>>>>>;

pub struct Finder {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
}

impl Finder {
    pub fn new(ipfs_client: IpfsClient) -> Self {
        Finder {
            ipfs_client,
            http_client: reqwest::Client::new(),
        }
    }

    /// Endpoint must serve operator info and the requested file
    async fn bundle_availability(
        &self,
        bundle_hash: &str,
        url: &str,
    ) -> Result<IndexerEndpoint, Error> {
        let files = self.indexer_status(url).await?;
        let operator: String = self.indexer_operator(url).await?;

        if !files.contains(&bundle_hash.to_string()) {
            tracing::trace!(
                url,
                files = tracing::field::debug(&files),
                "IPFS hash not found in served bundle status"
            );
            return Err(Error::DataUnavilable(format!(
                "IPFS hash not found in files served at {}",
                url
            )));
        }

        Ok((operator, url.to_string()))
    }

    /// Check the availability of a bundle at various indexer endpoints
    /// Return a list of endpoints where the desired bundle is hosted
    pub async fn bundle_availabilities(
        &self,
        bundle_hash: &str,
        endpoint_checklist: &[String],
    ) -> Vec<IndexerEndpoint> {
        tracing::debug!(bundle_hash, "Checking availability");

        // Use a stream to process the endpoints in parallel
        let results = stream::iter(endpoint_checklist)
            .map(|url| self.bundle_availability(bundle_hash, url))
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
        bundle_hash: &str,
        endpoint_checklist: &[String],
    ) -> Result<FileAvailbilityMap, Error> {
        let bundle = read_bundle(&self.ipfs_client, bundle_hash, PathBuf::new()).await?;
        // To fill in availability for each file, get a vector of (IndexerEndpoint, ManifestIPFS) that serves the file
        let target_hashes: FileAvailbilityMap = Arc::new(Mutex::new(
            bundle
                .file_manifests
                .iter()
                .map(|file_manifest| {
                    (
                        file_manifest.meta_info.hash.clone(),
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
        let bundles = self.indexer_status(url).await?;

        // Map of indexer_endpoints to served manifests
        // For each endpoint, populate indexer_map with the available files
        for bundle in bundles {
            let file_hashes: Vec<String> = fetch_bundle_from_ipfs(&self.ipfs_client, &bundle)
                .await?
                .files
                .iter()
                .map(|file| file.hash.clone())
                .collect();
            let file_map_lock = file_map.lock().await;
            for (target_file, availability_map) in file_map_lock.iter() {
                // Record serving indexer and bundle for each target file
                if file_hashes.contains(target_file) {
                    availability_map
                        .lock()
                        .await
                        .entry(indexer_endpoint.clone())
                        .and_modify(|e| e.push(bundle.clone()))
                        .or_insert(vec![bundle.clone()]);
                }
            }
        }

        match unavailable_files(&file_map).await {
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
pub async fn unavailable_files(file_map: &FileAvailbilityMap) -> Vec<String> {
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

//TODO: directly access the field instead
#[derive(Serialize, Deserialize)]
pub struct Operator {
    #[serde(alias = "publicKey")]
    pub public_key: String,
}