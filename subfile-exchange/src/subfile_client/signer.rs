use std::time::SystemTime;

use alloy_primitives::{Address, U256};
use alloy_sol_types::Eip712Domain;
use ethers::signers::Wallet;
use rand::RngCore;
use secp256k1::SecretKey;
use tap_core::eip_712_signed_message::EIP712SignedMessage;
use tap_core::tap_receipt::Receipt;

pub struct ReceiptSigner {
    signer: SecretKey,
    domain: Eip712Domain,
}

pub type SignedReceipt = EIP712SignedMessage<Receipt>;
pub trait Access {
    fn allocation(&self) -> Address;
    fn serialize(&self) -> String;
}

impl Access for SignedReceipt {
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

    pub async fn create_receipt(&self, allocation_id: Address, fee: u128) -> Option<SignedReceipt> {
        //TODO: need to get GRT typing (fee: &GRT) and Indexing typing for allocation
        // let allocation = *self.allocations.read().await.get(indexing)?;
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
            // value: fee.0.as_u128().unwrap_or(0),
            value: fee,
        };
        let wallet =
            Wallet::from_bytes(self.signer.as_ref()).expect("failed to prepare receipt wallet");
        let signed = EIP712SignedMessage::new(&self.domain, receipt, &wallet)
            .await
            .expect("failed to sign receipt");
        Some(signed)
    }
}
