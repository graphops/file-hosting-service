use std::time::SystemTime;

use alloy_primitives::{Address, U256};
use alloy_sol_types::Eip712Domain;
use ethers::signers::Wallet;
use rand::RngCore;
use secp256k1::SecretKey;
use tap_core::eip_712_signed_message::EIP712SignedMessage;
use tap_core::tap_receipt::Receipt;

use crate::errors::Error;
use crate::util::GRT;

pub struct ReceiptSigner {
    signer: SecretKey,
    domain: Eip712Domain,
}

pub type TapReceipt = EIP712SignedMessage<Receipt>;
pub trait Access {
    fn allocation(&self) -> Address;
    fn serialize(&self) -> String;
}

impl Access for TapReceipt {
    fn allocation(&self) -> Address {
        self.message.allocation_id
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

//TODO: would be different with TAPv1/2: add record_receipt, update_allocations, get_receipts
impl ReceiptSigner {
    pub async fn new(signer: SecretKey, chain_id: U256, verifier: Address) -> Self {
        Self {
            signer,
            domain: Eip712Domain {
                name: Some("TAP".into()),
                version: Some("1".into()),
                chain_id: Some(chain_id),
                verifying_contract: Some(verifier),
                salt: None,
            },
        }
    }

    pub async fn create_receipt(
        &self,
        allocation_id: Address,
        fee: &GRT,
    ) -> Result<ScalarReceipt, Error> {
        let nonce = rand::thread_rng().next_u64();
        let timestamp_ns = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .try_into()
            .unwrap();
        let receipt = Receipt {
            allocation_id,
            timestamp_ns,
            nonce,
            value: fee.0.as_u128().unwrap_or(0),
        };
        let wallet = Wallet::from_bytes(self.signer.as_ref()).map_err(Error::WalletError)?;
        EIP712SignedMessage::new(&self.domain, receipt, &wallet)
            .await
            .map_err(|e| Error::ContractError(e.to_string()))
            .map(ScalarReceipt::TAP)
    }
}

pub enum ScalarReceipt {
    TAP(EIP712SignedMessage<Receipt>),
}

impl ScalarReceipt {
    pub fn allocation(&self) -> Address {
        match self {
            ScalarReceipt::TAP(receipt) => receipt.message.allocation_id,
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            ScalarReceipt::TAP(receipt) => serde_json::to_string(&receipt).unwrap(),
        }
    }
}
