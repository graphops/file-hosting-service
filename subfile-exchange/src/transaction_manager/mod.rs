use ethers::contract::Contract;
use ethers::prelude::*;
use ethers_core::k256::ecdsa::SigningKey;

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use crate::errors::Error;

pub type NetworkContracts =
    HashMap<String, Contract<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>;
pub type ContractAddresses = HashMap<String, H160>;
pub type ContractClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TransactionManager {
    client: Arc<ContractClient>,
    contracts: NetworkContracts,
}

abigen!(
    L2Staking,
    "abis/L2Staking.json",
    // "npm:@graphprotocol/contracts@latest/dist/abis/L2Staking.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// Test function to simply call a read fn of a contract
async fn controller(client: &ContractClient, contract_addr: H160) -> Result<H160, Error> {
    let contract = L2Staking::new(contract_addr, Arc::new(client.clone()));

    let value = contract
        .controller()
        .call()
        .await
        .map_err(|e| Error::ContractError(e.to_string()))?;

    Ok(value)
}

impl TransactionManager {
    // Constructor to create a new instance
    pub async fn new(
        provider_url: &str,
        wallet: Wallet<SigningKey>,
    ) -> Result<Self, anyhow::Error> {
        let provider = Provider::<Http>::try_from(provider_url)?;
        let chain_id = provider.get_chainid().await?;
        let client = Arc::new(SignerMiddleware::new(provider, wallet));

        // Access contracts for the specified chain_id
        let contract_addresses =
            network_contract_addresses("addresses.json", &chain_id.to_string())?;

        // Initiate contract instances
        let contracts = NetworkContracts::new();
        // Test reading the function
        let _ = controller(&client, *contract_addresses.get("L2Staking").unwrap()).await?;

        Ok(TransactionManager { client, contracts })
    }
}

fn network_contract_addresses(
    file_path: &str,
    chain_id: &str,
) -> Result<ContractAddresses, anyhow::Error> {
    let data = fs::read_to_string(file_path)?;
    let json_value: Value = serde_json::from_str(&data)?;
    let mut network_contracts = ContractAddresses::new();

    if let Value::Object(chains) = json_value {
        if let Some(Value::Object(contracts)) = chains.get(chain_id) {
            for (contract_name, info) in contracts {
                if let Value::Object(info_map) = info {
                    if let Some(Value::String(address)) = info_map.get("address") {
                        network_contracts.insert(contract_name.clone(), H160::from_str(address)?);
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
