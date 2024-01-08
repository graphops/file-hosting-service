use ethers::contract::{abigen, Contract};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::coins_bip39::English;
use ethers::signers::{Signer, Wallet};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::{Bytes, TransactionReceipt, H160, U256};
use ethers_core::utils::keccak256;
use hdwallet::{DefaultKeyChain, ExtendedPrivKey};

use std::collections::HashMap;
use std::str::FromStr;

use crate::errors::Error;
use crate::transaction_manager::coins_bip39::Mnemonic;
use crate::util::{build_wallet, derive_key_pair};

use super::TransactionManager;

pub type NetworkContracts =
    HashMap<String, Contract<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>;
pub type ContractAddresses = HashMap<String, H160>;
pub type ContractClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

abigen!(
    L2Staking,
    "abis/L2Staking.json",
    // "npm:@graphprotocol/contracts@latest/abis/L2Staking.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl TransactionManager {
    /// Test function to simply call a read fn of a contract
    pub async fn controller(&self) -> Result<H160, Error> {
        let value = self
            .staking_contract
            .controller()
            .call()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;

        Ok(value)
    }

    /// call staking contract allocate function
    pub async fn allocate(
        &self,
        deployment: &str,
        tokens: U256,
        epoch: u64,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        //TODO: Start with hardcoding, later add field indexer address to TX manager, tokens to fn params
        let existing_ids: Vec<H160> = vec![];
        let metadata: [u8; 32] = [0; 32];

        let (allocation_signer, allocation_id) = unique_allocation_id(
            &self.args.mnemonic.clone(),
            epoch,
            deployment,
            &existing_ids,
        )?;
        let deployment_byte32 = ipfs_hash_to_bytes(deployment)?;
        let indexer_address = build_wallet(&self.args.mnemonic.clone())?.address();
        let proof = allocation_id_proof(&allocation_signer, indexer_address, allocation_id).await?;

        tracing::info!(
            dep_bytes = tracing::field::debug(&deployment_byte32),
            tokens = tracing::field::debug(&self.args.action),
            allocation_id = tracing::field::debug(&allocation_id),
            metadata = tracing::field::debug(&metadata),
            proof = tracing::field::debug(&proof),
            "allocate params",
        );

        let populated_tx = self.staking_contract.allocate(
            deployment_byte32,
            tokens,
            allocation_id,
            metadata,
            proof,
        );
        let estimated_gas = populated_tx
            .estimate_gas()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;
        tracing::debug!(
            estimated_gas = tracing::field::debug(&estimated_gas),
            "estimate gas"
        );

        // Attempt to send the populated tx with estimated gas, can later add a slippage
        let tx_result = populated_tx
            .gas(estimated_gas)
            .send()
            .await
            .map_err(|e| {
                let encoded_error = &e.to_string()[2..];
                let error_message_hex = &encoded_error[8 + 64..];
                let bytes = hex::decode(error_message_hex).unwrap();
                let message = String::from_utf8(bytes).unwrap();
                tracing::error!(message);

                Error::ContractError(e.to_string())
            })?
            .await
            .map_err(|e| {
                let encoded_error = &e.to_string()[2..];
                let error_message_hex = &encoded_error[8 + 64..];
                let bytes = hex::decode(error_message_hex).unwrap();
                let message = String::from_utf8(bytes).unwrap();
                tracing::error!(message);

                Error::ContractError(e.to_string())
            })?;
        tracing::debug!(
            value = tracing::field::debug(&tx_result),
            "allocate call result"
        );
        Ok((allocation_id, tx_result))
    }

    /// call staking contract allocate function
    pub async fn get_allocation(&self, allocation_id: H160) -> Result<Allocation, Error> {
        let value = self
            .staking_contract
            .get_allocation(allocation_id)
            .call()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;
        tracing::info!(value = tracing::field::debug(&value), "allocate call value");

        Ok(value)
    }

    /// call staking contract unallocate function
    pub async fn unallocate(
        &self,
        allocation_id: &str,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        let zero_poi: [u8; 32] = [0; 32];
        let allocation_id =
            H160::from_str(allocation_id).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        tracing::info!(
            allocation_id = tracing::field::debug(&allocation_id),
            "unallocate params",
        );

        let populated_tx = self
            .staking_contract
            .close_allocation(allocation_id, zero_poi);
        let estimated_gas = populated_tx
            .estimate_gas()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;
        tracing::debug!(
            estimated_gas = tracing::field::debug(&estimated_gas),
            "estimate gas"
        );

        // Attempt to send the populated tx with estimated gas, can later add a slippage
        let tx_result = populated_tx
            .gas(estimated_gas)
            .send()
            .await
            .map_err(|e| {
                let encoded_error = &e.to_string()[2..];
                let error_message_hex = &encoded_error[8 + 64..];
                let bytes = hex::decode(error_message_hex).unwrap();
                let message = String::from_utf8(bytes).unwrap();
                tracing::error!(message);

                Error::ContractError(e.to_string())
            })?
            .await
            .map_err(|e| {
                let encoded_error = &e.to_string()[2..];
                let error_message_hex = &encoded_error[8 + 64..];
                let bytes = hex::decode(error_message_hex).unwrap();
                let message = String::from_utf8(bytes).unwrap();
                tracing::error!(message);

                Error::ContractError(e.to_string())
            })?;
        tracing::debug!(
            value = tracing::field::debug(&tx_result),
            "allocate call result"
        );
        Ok((allocation_id, tx_result))
    }
}

/// create packed keccak hash for allocation id as proof
pub async fn allocation_id_proof(
    signer: &Wallet<SigningKey>,
    indexer_address: H160,
    allocation_id: H160,
) -> Result<Bytes, Error> {
    // Convert the raw bytes to hex and concatenate
    let combined_hex = format!(
        "{}{}",
        hex::encode(indexer_address.as_bytes()),
        hex::encode(allocation_id.as_bytes())
    );
    let bytes =
        hex::decode(combined_hex.clone()).map_err(|e| Error::InvalidConfig(e.to_string()))?;
    // Compute the Keccak-256 hash of the bytes
    let message_hash = keccak256(bytes);
    // Sign the message hash
    let signature = signer
        .sign_message(message_hash)
        .await
        .map_err(Error::WalletError)?;

    Ok(Bytes::from(signature.to_vec()))
}

/// Convert IPFS hash to byte representation
fn ipfs_hash_to_bytes(deployment: &str) -> Result<[u8; 32], Error> {
    let decoded_bytes = &bs58::decode(deployment)
        .into_vec()
        .map_err(|e| Error::InvalidConfig(format!("Failed to decode Qm Hash to bytes: {}", e)))?
        [2..];
    let mut deployment_byte32 = [0u8; 32];
    let len = std::cmp::min(decoded_bytes.len(), 32);
    deployment_byte32[..len].copy_from_slice(&decoded_bytes[..len]);
    let _hex_string = format!("0x{}", hex::encode(deployment_byte32));

    let decoded_bytes = bs58::decode(deployment)
        .into_vec()
        .map_err(|e| Error::InvalidConfig(format!("Failed to decode Qm Hash to bytes: {}", e)))?;
    if decoded_bytes.len() - 2 != 32 {
        return Err(Error::InvalidConfig(
            "Decoded bytes (minus the first two) are not 32 bytes long".into(),
        ));
    }

    Ok(deployment_byte32)
}

// Find a unique allocation ID for the indexer, epoch, and deployment
// take wallet mnemonic and derive address
// take epoch and deployment for a child signer
// filter from existing ids to ensure uniqueness
fn unique_allocation_id(
    mnemonic: &str,
    epoch: u64,
    deployment: &str,
    existing_ids: &[H160],
) -> Result<(Wallet<SigningKey>, H160), Error> {
    let seed = Mnemonic::<English>::from_str(mnemonic)
        .map_err(|e| Error::InvalidConfig(e.to_string()))?
        .to_seed(None)
        .map_err(|e| Error::InvalidConfig(e.to_string()))?;

    let key_chain = DefaultKeyChain::new(
        ExtendedPrivKey::with_seed(&seed).map_err(|e| Error::InvalidConfig(e.to_string()))?,
    );

    for i in 0..100 {
        let (private_key, address) = derive_key_pair(&key_chain, epoch, deployment, i)?;
        if !existing_ids.contains(&address) {
            let wallet = Wallet::from_str(&private_key).map_err(Error::WalletError)?;
            return Ok((wallet, address));
        }
    }

    Err(Error::ContractError("Exhausted limit of 100 allocations at the same time (This should be removed as allocation parallelization is deprecated)".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_allocation_id() {
        let indexer_mnemonic =
            "sheriff obscure trick beauty army fat wink legal flee leader section suit";
        let epoch: u64 = 0;
        let deployment = "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v";
        let mut existing_ids: Vec<H160> = vec![];

        for _i in 0..100 {
            let (_, allocation_id) =
                unique_allocation_id(indexer_mnemonic, epoch, deployment, &existing_ids).unwrap();
            existing_ids.push(allocation_id);
        }

        assert!(existing_ids.len() == 100);

        let uniquesness = unique_allocation_id(indexer_mnemonic, epoch, deployment, &existing_ids);
        assert!(uniquesness.is_err());
    }

    #[test]
    fn test_subgraph_deployment_repr() {
        let ipfs_hash = "QmWAsLViTdCbs9zbejzmRndpZpNXU97CzeLJwdZKuvCUdF";
        let expected = "745bf7153ea3c7d2cf7985042813945cd7797afabfbcf432eb0718a9dbead00a";
        assert!(hex::encode(ipfs_hash_to_bytes(ipfs_hash).unwrap()) == expected);
    }

    #[tokio::test]
    async fn test_allocation_id_proof() {
        let mnemonic = "sheriff obscure trick beauty army fat wink legal flee leader section suit";
        let epoch: u64 = 1024;
        let deployment = "QmWAsLViTdCbs9zbejzmRndpZpNXU97CzeLJwdZKuvCUdF";

        let indexer_address = build_wallet(mnemonic).unwrap().address();
        let existing_ids: Vec<H160> = vec![];
        let (allocation_signer, allocation_id) =
            unique_allocation_id(mnemonic, epoch, deployment, &existing_ids).unwrap();

        let id = format!("{:#?}", allocation_id);
        assert_eq!(id, "0x1cf8e1a42860cf19606da7e358f23d265b9ba6aa");

        let proof = allocation_id_proof(&allocation_signer, indexer_address, allocation_id)
            .await
            .unwrap();
        assert!(proof.to_string() == "0x1457a0a1a6a0531181bc31e8ed4c1dc9129f4dbeb8866b4213045ca275d14a757639ee5886c6778555d0b516c5776bbb919efc9cfbafd6f1017253e48b3d76301c")
        //With a secret mnemonic
        // assert_eq!(id, "0x6a39615c9f35ef68b4a99584c0566a1fcdd67d0a");
        // assert!(proof.to_string() == "0x7f47ee3a4e614c43352f8920e81371b0aa2298bb99ed6fc8eb4ed54f0bb1954c1463abd7a86a21185671f9a827c5ecaa1f509320cf9ee53f77338475018d1d931c")
    }
}
