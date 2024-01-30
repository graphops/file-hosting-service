use ethers::contract::abigen;

use ethers_core::types::{TransactionReceipt, H160, U256};

use crate::errors::Error;
use crate::transaction_manager::contract_error_decode;

use super::TransactionManager;

abigen!(
    L2GraphToken,
    "abis/L2GraphToken.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl TransactionManager {
    /// Test function to simply call a read fn of a contract
    pub async fn symbol(&self) -> Result<String, Error> {
        let value = self
            .token_contract
            .symbol()
            .call()
            .await
            .map_err(|e| Error::ContractError(e.to_string()))?;

        Ok(value)
    }

    // Approve spender and amount
    /// call staking contract allocate function
    pub async fn approve_escrow(
        &self,
        amount: &U256,
    ) -> Result<(H160, Option<TransactionReceipt>), Error> {
        let spender = self.contract_addresses.get("Escrow").unwrap();
        let populated_tx = self.token_contract.approve(*spender, *amount);
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
        Ok((*spender, tx_result))
    }
}
