use ethers::contract::{abigen, Contract};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::Wallet;
use ethers_core::{
    k256::ecdsa::SigningKey,
    types::{TransactionReceipt, H160, U256},
};

use std::collections::HashMap;

use crate::errors::Error;
use crate::transaction_manager::contract_error_decode;

use super::TransactionManager;

pub type NetworkContracts =
    HashMap<String, Contract<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>;
pub type ContractAddresses = HashMap<String, H160>;
pub type ContractClient = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

abigen!(
    Escrow,
    "abis/Escrow.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl TransactionManager {
    /// Test function to simply call a read fn of a contract
    pub async fn tap_verifier(&self) -> Result<H160, Error> {
        let value = self
            .escrow_contract
            .tap_verifier()
            .call()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;

        Ok(value)
    }

    /// Test function to simply call a read fn of a contract
    pub async fn escrow_amount(&self, sender: H160, receiver: H160) -> Result<U256, Error> {
        let value = self
            .escrow_contract
            .get_escrow_amount(sender, receiver)
            .call()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;

        Ok(value)
    }

    /// call staking contract allocate function
    pub async fn deposit(
        &self,
        receiver: &H160,
        tokens: &U256,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        let populated_tx = self.escrow_contract.deposit(*receiver, *tokens);
        let estimated_gas = populated_tx
            .estimate_gas()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            estimated_gas = tracing::field::debug(&estimated_gas),
            "estimate gas"
        );

        // Attempt to send the populated tx with estimated gas, can later add a slippage
        let tx_result = populated_tx
            .gas(estimated_gas)
            .send()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            value = tracing::field::debug(&tx_result),
            "deposit call result"
        );
        Ok((*receiver, tx_result))
    }

    /// call escrow contract deposit function
    pub async fn deposit_many(
        &self,
        receivers: Vec<H160>,
        tokens: Vec<U256>,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        let populated_tx = self.escrow_contract.deposit_many(receivers, tokens);
        let estimated_gas = populated_tx
            .estimate_gas()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            estimated_gas = tracing::field::debug(&estimated_gas),
            "estimate gas"
        );

        // Attempt to send the populated tx with estimated gas, can later add a slippage
        let tx_result = populated_tx
            .gas(estimated_gas)
            .send()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            value = tracing::field::debug(&tx_result),
            "depositMany call result"
        );
        Ok((H160::default(), tx_result))
    }

    /// call escrow contract withdraw function
    pub async fn withdraw(
        &self,
        receiver: &H160,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        let populated_tx = self.escrow_contract.withdraw(*receiver);
        let estimated_gas = populated_tx
            .estimate_gas()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            estimated_gas = tracing::field::debug(&estimated_gas),
            "estimate gas"
        );

        // Attempt to send the populated tx with estimated gas, can later add a slippage
        let tx_result = populated_tx
            .gas(estimated_gas)
            .send()
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?
            .await
            .map_err(|e| Error::ContractError(contract_error_decode(e.to_string())))?;
        tracing::debug!(
            value = tracing::field::debug(&tx_result),
            "withdraw call result"
        );
        Ok((*receiver, tx_result))
    }
}
