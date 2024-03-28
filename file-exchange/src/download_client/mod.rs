use alloy_primitives::Address;

use ethers::providers::{Http, Middleware, Provider};

use ethers_core::types::{H160, U256};
use rand::seq::SliceRandom;
use reqwest::header::{HeaderName, AUTHORIZATION};
use secp256k1::SecretKey;
use std::collections::{HashMap, HashSet};
use std::fs::File;

use std::fs;
use std::io::Read;
use std::ops::Sub;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex as StdMutex};

use tokio::sync::Mutex;

use crate::util::{read_json_to_map, store_map_as_json};
use crate::{
    config::{DownloaderArgs, OnChainArgs, StorageMethod},
    discover::{Finder, ServiceEndpoint},
    download_client::range_request::download_chunk_and_write_to_file,
    errors::Error,
    graphql::{allocation_id, escrow_query::escrow_balance},
    manifest::{
        ipfs::IpfsClient, manifest_fetcher::read_bundle, store::Store, Bundle, FileManifestMeta,
    },
    transaction_manager::TransactionManager,
    util::build_wallet,
};

use self::{range_request::DownloadRangeRequest, signer::ReceiptSigner};

pub mod range_request;
pub mod signer;

pub struct Downloader {
    config: DownloaderArgs,
    http_client: reqwest::Client,
    bundle: Bundle,
    _gateway_url: Option<String>,
    indexer_urls: Arc<StdMutex<Vec<ServiceEndpoint>>>,
    indexer_blocklist: Arc<StdMutex<HashSet<String>>>,
    // key is the file manifest identifier (IPFS hash) and value is a HashSet of downloaded chunk indices
    pub target_chunks: Arc<StdMutex<HashMap<String, HashSet<u64>>>>,
    bundle_finder: Finder,
    payment: PaymentMethod,
    store: Store,
}

/// A downloader can either provide a free query auth token or receipt signer
pub enum PaymentMethod {
    FreeQuery(String),
    PaidQuery(OnChainSigner),
}

pub struct OnChainSigner {
    #[allow(dead_code)]
    transaction_manager: TransactionManager,
    receipt_signer: ReceiptSigner,
}

impl Downloader {
    pub async fn new(ipfs_client: IpfsClient, args: DownloaderArgs) -> Self {
        let bundle = read_bundle(&ipfs_client, &args.ipfs_hash)
            .await
            .expect("Read bundle");

        let payment = if let Some(token) = &args.free_query_auth_token {
            PaymentMethod::FreeQuery(token.clone())
        } else if let Some(mnemonic) = &args.mnemonic {
            let wallet = build_wallet(mnemonic).expect("Mnemonic build wallet");
            let signing_key = wallet.signer().to_bytes();
            let secp256k1_private_key =
                SecretKey::from_slice(&signing_key).expect("Private key from wallet");
            let provider_link = args
                .provider
                .as_ref()
                .expect("Provider required to connect")
                .clone();
            let provider =
                Provider::<Http>::try_from(&provider_link).expect("Connect to the provider");
            //TODO: migrate ethers type to alloy
            let chain_id = alloy_primitives::U256::from(
                provider
                    .get_chainid()
                    .await
                    .expect("Get chain id from provider")
                    .as_u128(),
            );
            let transaction_manager = TransactionManager::new(OnChainArgs {
                action: None,
                mnemonic: mnemonic.to_string(),
                provider: provider_link.clone(),
                verifier: args.verifier.clone(),
                network_subgraph: args.network_subgraph.clone(),
                escrow_subgraph: args.escrow_subgraph.clone(),
            })
            .await
            .expect("Initialize transaction manager for paid queries");
            let receipt_signer = ReceiptSigner::new(
                secp256k1_private_key,
                chain_id,
                Address::from_str(args.verifier.as_ref().expect("Provide verifier"))
                    .expect("Parse verifier"),
            )
            .await;
            PaymentMethod::PaidQuery(OnChainSigner {
                transaction_manager,
                receipt_signer,
            })
        } else {
            panic!("No payment wallet nor free query token provided");
        };

        let store = Store::new(&args.storage_method).expect("Create store");

        let target_chunks = if let Some(file_path) = &args.progress_file {
            let map = read_json_to_map(file_path).expect("Progress cache ill-formatted");
            Arc::new(StdMutex::new(map))
        } else {
            Arc::new(StdMutex::new(HashMap::new()))
        };

        Downloader {
            config: args.clone(),
            http_client: reqwest::Client::new(),
            bundle,
            _gateway_url: args.gateway_url,
            indexer_urls: Arc::new(StdMutex::new(Vec::new())),
            indexer_blocklist: Arc::new(StdMutex::new(HashSet::new())),
            target_chunks,
            bundle_finder: Finder::new(ipfs_client),
            payment,
            store,
        }
    }

    pub fn update_indexer_urls(&self, endpoints: &[ServiceEndpoint]) {
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
    pub fn init_target_chunks(&self, bundle: &Bundle) {
        if let Some(file_path) = &self.config.progress_file {
            let mut target_chunks = self.target_chunks.lock().unwrap();
            *target_chunks = read_json_to_map(file_path).expect("Progress cache ill-formatted");
        }
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
    pub async fn download_bundle(&self) -> Result<(), Error> {
        self.init_target_chunks(&self.bundle);
        tracing::trace!(
            chunks = tracing::field::debug(self.target_chunks.clone()),
            "File manifests download starting"
        );

        // check bundle availability from gateway/indexer_endpoints
        self.availbility_check().await?;
        // check balance availability if payment is enabled
        self.escrow_check().await?;

        // Loop through file manifests for downloading
        let mut incomplete_progresses = HashMap::new();
        for file_manifest in &self.bundle.file_manifests {
            if let Err(e) = self.download_file_manifest(file_manifest.clone()).await {
                tracing::warn!(
                    hash = &file_manifest.meta_info.hash,
                    error = e.to_string(),
                    "Failed to download file"
                );
                incomplete_progresses.insert(
                    file_manifest.meta_info.hash.clone(),
                    self.remaining_chunks(&file_manifest.meta_info.hash)
                        .into_iter()
                        .collect(),
                );
            }
        }

        if !incomplete_progresses.is_empty() {
            let msg = format!(
                "File manifests download incomplete: {:#?}; Store progress for next attempt",
                tracing::field::debug(&incomplete_progresses),
            );
            tracing::warn!(msg);
            // store progress into a json file: {hash: missing_chunk_indices}
            if let Some(file_path) = &self.config.progress_file {
                store_map_as_json(&incomplete_progresses, file_path)?;
            };
            return Err(Error::DataUnavailable(msg));
        }

        tracing::info!("File manifests download completed");

        if let Some(file_path) = &self.config.progress_file {
            let _ = fs::remove_file(file_path);
        };

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
    pub async fn download_file_manifest(&self, meta: FileManifestMeta) -> Result<(), Error> {
        tracing::debug!(
            file_spec = tracing::field::debug(&meta),
            "Download file manifest"
        );

        // If storage method is the local file system, directly write the ranges
        // If remote object storage, first write ranges to a tmp file to complete the object
        // Open the output file
        let file = match &self.store.storage_method {
            StorageMethod::LocalFiles(directory) => {
                let file = File::create(Path::new(
                    &(directory.main_dir.clone() + "/" + &meta.meta_info.name),
                ))
                .unwrap_or_else(|_| {
                    panic!(
                        "Cannot create file for writing the output at directory {}",
                        &directory.main_dir
                    )
                });

                Arc::new(Mutex::new(file))
            }
            StorageMethod::ObjectStorage(store) => {
                let file = File::create(Path::new(
                    &("tmp/".to_owned() + &store.bucket + "/" + &meta.meta_info.name),
                ))
                .unwrap_or_else(|_| {
                    panic!(
                        "Cannot create file for writing the output at tmp/{}",
                        &store.bucket.clone()
                    )
                });
                tracing::debug!("Created tmp directory");
                Arc::new(Mutex::new(file))
            }
        };

        while !self.remaining_chunks(&meta.meta_info.hash).is_empty() {
            // Wait for all chunk tasks to complete and collect the results
            let mut handles = Vec::new();
            for i in self.remaining_chunks(&meta.meta_info.hash) {
                let file_manifest_hash = meta.meta_info.hash.clone();
                let client = self.http_client.clone();
                //TODO: can utilize operator address for on-chain checks
                let request = self.download_range_request(&meta, i, file.clone())?;
                let payment = self.payment_header(&request.receiver).await?;
                let block_list = self.indexer_blocklist.clone();
                let target_chunks: Arc<StdMutex<HashMap<String, HashSet<u64>>>> =
                    self.target_chunks.clone();
                let url = request.query_endpoint.clone();
                // Spawn a new asynchronous task for each range request
                let handle: tokio::task::JoinHandle<Result<Arc<Mutex<File>>, Error>> =
                    tokio::spawn(async move {
                        match download_chunk_and_write_to_file(&client, request, payment).await {
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
                                    url,
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
                    .map_err(|e| Error::DataUnavailable(e.to_string()))?;
            }
        }

        tracing::info!(
            chunks = tracing::field::debug(self.target_chunks.clone()),
            file_info = tracing::field::debug(&meta.meta_info),
            "File finished"
        );
        // If remote storage is configured, write to remote store and clean temp
        if let StorageMethod::ObjectStorage(store) = &self.store.storage_method {
            let file_path = &("tmp/".to_owned() + &store.bucket + "/" + &meta.meta_info.name);
            let bytes = read_file_contents(file_path).await?;
            let write_id = self
                .store
                .multipart_write(&meta.meta_info.name, &bytes, None)
                .await;

            tracing::debug!("Wrote with id {write_id:?}; delete tmp file");
            fs::remove_file(file_path).map_err(Error::FileIOError)?;
        };
        Ok(())
    }

    /// Make a header for chunk request authorization either free or paid
    async fn payment_header(&self, receiver: &str) -> Result<(HeaderName, String), Error> {
        match &self.payment {
            PaymentMethod::FreeQuery(token) => Ok((AUTHORIZATION, token.to_string())),
            PaymentMethod::PaidQuery(signer) => {
                let receipt = signer
                    .receipt_signer
                    .create_receipt(allocation_id(receiver), &Finder::fees())
                    .await?;
                Ok((
                    // HeaderName::from_str("Scalar-Receipt").unwrap(),
                    HeaderName::from_str("scalar-receipt").unwrap(),
                    receipt.serialize(),
                ))
            }
        }
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
        let blocklist = self
            .indexer_blocklist
            .lock()
            .map_err(|e| Error::DataUnavailable(format!("Cannot unwrap indexer_blocklist: {}", e)))?
            .clone();
        tracing::debug!(blocklist = tracing::field::debug(&blocklist), "blocklist");
        let filtered_endpoints = query_endpoints
            .iter()
            .filter(|url| !blocklist.contains(&url.service_endpoint))
            .cloned()
            .collect::<Vec<_>>();
        let service = if let Some(service) = filtered_endpoints.choose(&mut rng).cloned() {
            tracing::debug!(
                service = tracing::field::debug(&service),
                chunk = i,
                file_manifest = meta.meta_info.hash,
                "Randomly picked provider"
            );
            service
        } else {
            let err_msg = "No operator serving the file, data unavailable".to_string();
            tracing::warn!(err_msg);
            return Err(Error::DataUnavailable(err_msg.to_string()));
        };
        //TODO: do not add ipfs_hash here, construct query_endpoint after updating route 'files/id/:id'
        let query_endpoint =
            service.service_endpoint.clone() + "/files/id/" + &self.config.ipfs_hash;
        let file_hash = meta.meta_info.hash.clone();
        let start = i * meta.file_manifest.chunk_size;
        let end = u64::min(
            start + meta.file_manifest.chunk_size,
            meta.file_manifest.total_bytes,
        ) - 1;
        let chunk_hash = meta.file_manifest.chunk_hashes[i as usize].clone();

        Ok(DownloadRangeRequest {
            receiver: service.operator.clone(),
            query_endpoint,
            file_hash,
            start,
            end,
            chunk_hash,
            file,
            max_retry: self.config.max_retry,
        })
    }

    /// Make sure the requested bundle is available from at least 1 provider
    async fn availbility_check(&self) -> Result<(), Error> {
        let blocklist = self.indexer_blocklist.lock().unwrap().clone();
        let endpoints = &self
            .config
            .indexer_endpoints
            .iter()
            .filter(|url| !blocklist.contains(*url))
            .cloned()
            .collect::<Vec<_>>();
        let all_available = &self
            .bundle_finder
            .bundle_availabilities(&self.config.ipfs_hash, endpoints)
            .await;
        let mut sorted_endpoints = all_available.to_vec();
        // Sort by price_per_byte in ascending order and select the top 'provider_concurrency' endpoints
        //TODO: add other types of selection such as latency and reliability
        sorted_endpoints.sort_by(|a, b| {
            a.price_per_byte
                .partial_cmp(&b.price_per_byte)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.update_indexer_urls(
            &sorted_endpoints
                .into_iter()
                .take(self.config.provider_concurrency as usize)
                .collect::<Vec<ServiceEndpoint>>(),
        );
        let indexer_endpoints = self.indexer_urls.lock().unwrap().clone();
        if indexer_endpoints.is_empty() {
            tracing::warn!(
                bundle_hash = &self.config.ipfs_hash,
                "No endpoint satisfy the bundle requested, sieve through available bundles for individual files"
            );

            // check files availability from gateway/indexer_endpoints
            match self
                .bundle_finder
                .file_discovery(&self.config.ipfs_hash, endpoints)
                .await
            {
                Ok(map) => {
                    let msg = format!(
                        "Files available on these available bundles: {:#?}",
                        tracing::field::debug(&map.lock().await),
                    );
                    return Err(Error::DataUnavailable(msg));
                }
                Err(e) => {
                    let msg = format!(
                        "Cannot match the files: {:?}, {:?}",
                        tracing::field::debug(&self.indexer_urls.lock().unwrap()),
                        tracing::field::debug(&e),
                    );
                    tracing::error!(msg);
                    return Err(Error::DataUnavailable(msg));
                }
            }
        };
        Ok(())
    }

    /// Check escrow balances with cheapest N providers (N is the downloader client configured provider concurrency)
    /// Make suggestion to individual escrow accounts if balance is low
    /// Error out if gross buying power is insufficient, otherwise proceed with downloading
    async fn escrow_check(&self) -> Result<(), Error> {
        // check balance availability if payment is enabled
        tracing::trace!("Escrow account checks");
        if let PaymentMethod::PaidQuery(on_chain) = &self.payment {
            let fail_tolerance = 1.2_f64;

            let mut total_buying_power_in_bytes: f64 = 0.0;
            // estimate the cost to download the bundle from each provider
            let total_bytes = self
                .bundle
                .file_manifests
                .iter()
                .map(|f| f.file_manifest.total_bytes)
                .sum::<u64>();

            let endpoints = self.indexer_urls.lock().unwrap().clone();
            let multiplier = (total_bytes as f64)
                / ((self.config.provider_concurrency).min(endpoints.len().try_into().unwrap())
                    as f64)
                * fail_tolerance;
            let mut insufficient_balances = vec![];
            for endpoint in endpoints {
                tracing::trace!(
                    endpoint = tracing::field::debug(&endpoint),
                    "Check escrow account for indexer"
                );
                let escrow_requirement = endpoint.price_per_byte * multiplier;
                let escrow = &on_chain.transaction_manager.args.escrow_subgraph;
                let sender = on_chain.transaction_manager.public_address()?;
                let receiver = endpoint.operator;
                let account = escrow_balance(&self.http_client, escrow, &sender, &receiver).await?;

                // check for escrow balance
                let missing = match account {
                    None => {
                        tracing::warn!("Escrow account doesn't exist, make deposits");
                        escrow_requirement
                    }
                    Some(a) if a.lt(&escrow_requirement) => {
                        let msg = format!(
                            "Top-up required to make averaged concurrent requests: {} availble, recommend topping up to {}",
                            a, escrow_requirement
                        );
                        tracing::warn!(msg);
                        // buying power to a specific indexer is (escrow balance / indexer price = number of bytes)
                        total_buying_power_in_bytes += a / endpoint.price_per_byte;
                        escrow_requirement - a
                    }
                    Some(a) => {
                        total_buying_power_in_bytes += a / endpoint.price_per_byte;
                        tracing::trace!(
                            balance = a,
                            price_per_byte = endpoint.price_per_byte,
                            total_bytes,
                            buying_power_in_bytes = a / endpoint.price_per_byte,
                            "Balance is enough for this account"
                        );
                        0.0
                    }
                };
                insufficient_balances.push((receiver.clone(), missing));
            }

            let total_deficit = insufficient_balances.iter().map(|(_, v)| v).sum::<f64>();
            if (total_buying_power_in_bytes as u64) >= total_bytes {
                tracing::info!("Balance is enough to purchase the file, go ahead to download");
            } else if total_deficit <= self.config.max_auto_deposit {
                tracing::info!("Downloader is allowed to automatically deposit sufficient balance for complete download, now depositing");
                let escrow_allowance = f64::from_str(
                    &on_chain
                        .transaction_manager
                        .escrow_allowance()
                        .await?
                        .to_string(),
                )
                .map_err(|e| Error::ContractError(e.to_string()))?;
                if total_deficit.gt(&escrow_allowance) {
                    let missing_allowance = U256::from_dec_str(
                        &total_deficit
                            .sub(total_deficit.sub(escrow_allowance))
                            .to_string(),
                    )
                    .map_err(|e| Error::ContractError(e.to_string()))?;
                    let _ = on_chain
                        .transaction_manager
                        .approve_escrow(&missing_allowance)
                        .await?;
                };

                let deficits: Vec<(String, f64)> = insufficient_balances
                    .into_iter()
                    .filter(|&(_, amount)| amount != 0.0)
                    .collect();
                let (receivers, amounts): (Vec<String>, Vec<f64>) = deficits.into_iter().unzip();
                let tx_res = on_chain
                    .transaction_manager
                    .deposit_many(
                        receivers
                            .into_iter()
                            .map(|s| H160::from_str(&s).expect("Operator not address"))
                            .collect(),
                        amounts
                            .into_iter()
                            .map(|f| {
                                tracing::info!(
                                    amount = &f.ceil().to_string().to_string(),
                                    "amount"
                                );
                                U256::from_dec_str(&f.ceil().to_string())
                                    .expect("Amount not parseable")
                            })
                            .collect(),
                    )
                    .await;
                if let Err(e) = tx_res {
                    tracing::warn!(error = e.to_string(), "Failed to submit Escrow deposit, might need to approve Escrow contract as a GRT spender");

                    return Err(Error::ContractError(e.to_string()));
                };
                tracing::info!("Finished Escrow deposit, okay to initiate download");
            } else {
                let msg = format!("Balance is not enough to purchase {} bytes, look at the error message to see top-up recommendations for each account, or configure maximum automatic deposit threshold to be greater than the deficit amount of {}", total_bytes, total_deficit);
                return Err(Error::PricingError(msg));
            }
        };

        Ok(())
    }
}

/// extract base indexer_url from `indexer_url/bundles/id/bundle_id`
fn extract_base_url(query_endpoint: &str) -> Option<&str> {
    if let Some(index) = query_endpoint.find("/files/id/") {
        Some(&query_endpoint[..index])
    } else {
        None
    }
}

async fn read_file_contents(file: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(Path::new(file))
        .unwrap_or_else(|_| panic!("Cannot open file {} to transfer to object store", file));
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(Error::FileIOError)?;
    Ok(contents)
}
