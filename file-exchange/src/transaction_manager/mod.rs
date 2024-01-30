use ethers::prelude::*;
use ethers_core::k256::ecdsa::SigningKey;

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::WalletArgs;
use crate::errors::Error;
use crate::transaction_manager::{escrow::Escrow, graph_token::L2GraphToken, staking::L2Staking};
use crate::util::build_wallet;

pub mod escrow;
pub mod graph_token;
pub mod staking;

/// Contracts: (contract name, contract address)
pub type ContractAddresses = HashMap<String, H160>;
/// Client with provider endpoint and a wallet
pub type ContractClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TransactionManager {
    client: Arc<ContractClient>,
    contract_addresses: ContractAddresses,
    // TODO: refactor these; only initiate if called by the tx manager
    staking_contract: L2Staking<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
    escrow_contract: Escrow<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
    token_contract: L2GraphToken<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
    pub args: WalletArgs,
}

impl TransactionManager {
    // Constructor to create a new instance
    pub async fn new(args: WalletArgs) -> Result<Self, anyhow::Error> {
        tracing::info!("Initialize transaction manager");
        let provider = Provider::<Http>::try_from(&args.provider)?;
        let chain_id = provider.get_chainid().await?;
        let wallet = build_wallet(&args.mnemonic)
            .expect("Mnemonic build wallet")
            .with_chain_id(provider.get_chainid().await.unwrap().as_u64());
        let client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

        // Access contracts for the specified chain_id
        let contract_addresses =
            network_contract_addresses("addresses.json", &chain_id.to_string())?;

        let staking_addr = contract_addresses.get("L2Staking").unwrap();
        let staking_contract = L2Staking::new(*staking_addr, Arc::new(client.clone()));

        let escrow_addr = contract_addresses.get("Escrow").unwrap();
        let escrow_contract = Escrow::new(*escrow_addr, Arc::new(client.clone()));

        let token_addr = contract_addresses.get("L2GraphToken").unwrap();
        let token_contract = L2GraphToken::new(*token_addr, Arc::new(client.clone()));

        Ok(TransactionManager {
            client,
            contract_addresses,
            staking_contract,
            escrow_contract,
            token_contract,
            args,
        })
    }
}

/// Track network contract addresses given an address book in json
pub fn network_contract_addresses(
    file_path: &str,
    chain_id: &str,
) -> Result<ContractAddresses, Error> {
    let data = fs::read_to_string(file_path).map_err(|e| Error::ContractError(e.to_string()))?;
    let json_value: Value =
        serde_json::from_str(&data).map_err(|e| Error::ContractError(e.to_string()))?;
    let mut network_contracts = ContractAddresses::new();

    if let Value::Object(chains) = json_value {
        if let Some(Value::Object(contracts)) = chains.get(chain_id) {
            for (contract_name, info) in contracts {
                if let Value::Object(info_map) = info {
                    if let Some(Value::String(address)) = info_map.get("address") {
                        let addr = H160::from_str(address);
                        tracing::debug!("Read contract address {:#?} -> addr {:#?}", address, addr);
                        network_contracts.insert(
                            contract_name.clone(),
                            addr.map_err(|e| Error::ContractError(e.to_string()))?,
                        );
                    }
                }
            }
        }
    }

    tracing::debug!(
        network_contracts = tracing::field::debug(&network_contracts),
        "network"
    );
    Ok(network_contracts)
}

pub fn contract_error_decode(e: String) -> String {
    let encoded_error = &e.to_string()[2..];
    let error_message_hex = &encoded_error[8 + 64..];
    let bytes = hex::decode(error_message_hex).unwrap();
    let message = String::from_utf8(bytes).unwrap();
    tracing::error!(message);
    message
}
