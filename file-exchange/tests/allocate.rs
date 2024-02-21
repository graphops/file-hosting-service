#[cfg(test)]
mod tests {

    use chrono::Utc;
    use ethers_core::types::U256;
    use file_exchange::{
        config::{AllocateArgs, OnchainAction, WalletArgs},
        transaction_manager::TransactionManager,
    };

    #[tokio::test]
    #[ignore]
    async fn test_allocate() {
        // 1. Basic setup; const
        std::env::set_var("RUST_LOG", "off,file_exchange=debug,allocate=trace");
        file_exchange::config::init_tracing("pretty").unwrap();
        let wallet_args = WalletArgs {
            mnemonic: String::from(
                "sheriff obscure trick beauty army fat wink legal flee leader section suit",
            ),
            provider: String::from("https://arbitrum-sepolia.infura.io/v3/aaaaaaaaaaaaaaaaaaaa"),
            verifier: Some(String::from("0xfC24cE7a4428A6B89B52645243662A02BA734ECF")),
            network_subgraph: String::from("https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-arbitrum-sepolia"),
            action: Some(file_exchange::config::OnchainAction::Allocate(
                AllocateArgs {
                    tokens: U256::from(100),
                    deployment_ipfs: String::from("QmeKabcCQBtgU6QjM3rp3w6pDHFW4r54ee89nGdhuyDuhi"),
                    epoch: Utc::now().timestamp() as u64,
                },
            )),
        };

        let transaction_manager = TransactionManager::new(wallet_args).await.unwrap();

        // 2. Send allocate tx
        let action = transaction_manager.args.action.clone().unwrap();
        let allocate_args = if let OnchainAction::Allocate(args) = action {
            args.clone()
        } else {
            panic!("No allocate args")
        };
        let (allocation_id, tx_receipt) = transaction_manager
            .allocate(
                &allocate_args.deployment_ipfs,
                allocate_args.tokens,
                allocate_args.epoch,
            )
            .await
            .unwrap();
        tracing::trace!(
            allocation_id = tracing::field::debug(&allocation_id),
            tx_receipt = tracing::field::debug(&tx_receipt),
            "get_allocation call result"
        );

        // 3. Validate allocation
        let allocation = transaction_manager.get_allocation(allocation_id).await;
        tracing::trace!(
            allocation = tracing::field::debug(&allocation),
            "get_allocation call result"
        );

        assert!(allocation.is_ok());
    }
}
