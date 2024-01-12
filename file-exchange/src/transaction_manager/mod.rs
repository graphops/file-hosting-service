use ethers::prelude::*;
use ethers_core::k256::ecdsa::SigningKey;

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::WalletArgs;
use crate::transaction_manager::staking::L2Staking;
use crate::util::build_wallet;

pub mod staking;

/// Contracts: (contract name, contract address)
pub type ContractAddresses = HashMap<String, H160>;
/// Client with provider endpoint and a wallet
pub type ContractClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TransactionManager {
    client: Arc<ContractClient>,
    staking_contract: L2Staking<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
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

        // // Test reading the function
        // let tokens = U256::from(100000);
        // let _value = staking::allocate(&client, *contract_addresses.get("L2Staking").unwrap(), "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v", token).await?;

        // let value =
        //     staking::controller(&client, *contract_addresses.get("L2Staking").unwrap()).await?;
        // tracing::debug!("test read - controller value: {:#?}", value);

        let staking_addr = contract_addresses.get("L2Staking").unwrap();
        let staking_contract = L2Staking::new(*staking_addr, Arc::new(client.clone()));

        Ok(TransactionManager {
            client,
            staking_contract,
            args,
        })
    }
}

/// Track network contract addresses given an address book in json
pub fn network_contract_addresses(
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
