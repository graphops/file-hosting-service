use std::str::FromStr;

use alloy_primitives::Address;

// This file should contain graphql queries for
// network_subgraph: fetching registered_indexers' public_url, indexer file hosting allocation
// escrow_subgraph: escrow account(sender,receiver) balance

pub fn allocation_id(_indexer: &str) -> Address {
    Address::from_str("0x29cc405f6104b1d6d2d7f2989c5932818f6268c2").unwrap()
}
