# On-chain Interactions

## Contract specification

Contract artifacts are generated from https://github.com/graphprotocol/contracts at commit a667f7aba10d558af6485d34cf857204321e343c. (location: `/build/abis`). Contract address book (`addresses.jon`) is copy and pasted from the same commit.

> In theory, contract ABIs can be imported with a path using Etherscan or npmjs, with "online" / "abigen_online" feature. But having issues to connect with this method. The current manual solution is only temporary.

## Available CLI commands

[On-Chain Guide](onchain_guide.md)

The subcommand `wallet` is designed to facilitate various on-chain actions through a command-line interface. This guide focuses on how users can perform wallet transactions on the blockchain.

## Configuration

Before starting, ensure you have set up the following:

- **Mnemonic**: A secret phrase (mnemonic) for accessing your wallet.
- **Provider URL**: The endpoint URL of your blockchain provider. This is limited to an Ethereum mainnet provider, Arbitrum-One provider, or an Arbitrum Sepolia provider. 

## Usage

You are not required to use this command. 
- Downloader client can handle automatic deposits by configuring the `max-auto-deposit` parameter.
- Server will not need to create an explicit allocation for free query services at the moment. We expect to add automatic allocation management in relation to payments system later on. 

For manual on-chain interactions, the following subcommands and options for `file-exchange wallet` are available:

### Global Options

- `--mnemonic KEY`: Sets the mnemonic for the wallet. This is required for any wallet operations.
- `--provider provider_url`: The blockchain provider endpoint URL.

### Subcommands

Data producer

- `allocate`: Allocate stake for a particular file/bundle/deployment.
- `unallocate`: Close an allocation to stop service and collect rewards (0x0 POI is used since FHS doesn't plan on supporting indexing rewards).

Data consumer
- `approve`: **First time** Escrow user may need to first approve Escrow contract as spender for their GRT; specify an amount.
- `deposit`: Deposit tokens to a specific data producer; specify an amount less than the approved amount.
- `deposit_many`: Deposit tokens to a vector of data producers; specify a vector of amounts in order of the producers.
- `withdraw`: Withdraw tokens from Escrow deposit; specify the address of the producer.

#### Allocate

To open allocation towards a deployment, provide the deployment IPFS hash, the token amount, the current epoch number (should later be resolved automaically), and fill in the `allocate` subcommand with the necessary arguments:

```shell
✗ file-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    allocate \
        --tokens <tokens> \
        --deployment-ipfs <deployment_ipfs> \
        --epoch <epoch>
```

#### Unallocate

To close allocation with 0x0 PoI.

```shell
✗ file-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    allocate \
        --allocation-id <id>
```

#### Approve

First time Escrow contract depositer would need to first approve Escrow contract as a spender from the GraphToken contract. This command allows a convinent CLI for the downloaders to set an allowance; this is not a deposit, but merely an allowance for the future, I suggest setting a high amount. 

```shell
✗ file-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    approve \
        --tokens <tokens>
```

#### Deposit

To deposit tokens in the Escrow contract, provide a receiver of the tokens and the amount to store in escrow.

```shell
✗ file-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    deposit \
        --receiver <receiver_address> \
        --tokens <tokens>
```

**DepositMany** is similar, but taking vectors as an input with comma separation.  

#### Withdraw

Withdraw deposit from a particular receiver.

```shell
✗ file-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    withdraw \
        --receiver <receiver>
```

### Examples

- Replace placeholders like `"mnemonic phrase"`, `"http://localhost:8545"`, `"0x123..."`, `"QmHash"`, etc., with actual values.
- Ensure that the mnemonic and provider URL are kept secure and private.

**Allocating**

Grab the IPFS hash of the deployment you want to allocate to and decide the allocation amount. Note that you can only open 1 allocation per deployment per epoch.

```shell
✗ cargo run -p file-exchange wallet \
    --mnemonic "mnemonic phrase" \
    --provider "http://localhost:8545" \
    allocate \
        --tokens 100 \
        --deployment-ipfs QmHash \
        --epoch 100
```

With RUST_LOG turned on, you can expect the following logs upon success
```
  INFO file_exchange::transaction_manager: Initialize transaction manager
    at file-exchange/src/transaction_manager/mod.rs:32

  INFO file_exchange::transaction_manager::staking: allocate params, dep_bytes: [241, 64, 71, 78, 218, 63, 159, 91, 130, 173, 178, 168, 30, 254, 183, 20, 225, 131, 35, 230, 52, 85, 74, 196, 40, 255, 173, 61, 144, 126, 223, 33], tokens: Some(Allocate(AllocateArgs { tokens: 256, deployment_ipfs: "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v", 
  epoch: 101 })), allocation_id: 0x75e11e0f2319913c28d0b1916b4b1d9a1ac3977b, metadata: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], proof: Bytes(0xac1b36d68ea95ebe9f9793850cf083a031d40806121dc2dac525423c50611d18053f195627f6bffe036b9325b4dfd273959457b5d3f1c1b53095c096182756bb1b)
    at file-exchange/src/transaction_manager/staking.rs:67

  DEBUG file_exchange::transaction_manager::staking: estimate gas, estimated_gas: 379109
    at file-exchange/src/transaction_manager/staking.rs:82

  INFO file_exchange: Allocation transaction finished, allocation_id: 0x75e11e0f2319913c28d0b1916b4b1d9a1ac3977b, tx_receipt: Some(TransactionReceipt { transaction_hash: 0x835b790326abf1555545920265e54d5bfbaba150aef31820529736e6727c7a0a, ... })
    at file-exchange/src/main.rs:75
```

**Closing allocation**

Grab the ID of allocation you want to close, and populate the unallocate subcommand
```
✗ cargo run -p file-exchange wallet \
    --mnemonic "mnemonic" \
    --provider "provider_url" \
    unallocate --allocation-id 0xe37b9ee6d657ab5700e8a964a8fcc8b39cdefd73 
```

You can expect logs as follows
```
  INFO file_exchange::transaction_manager: Initialize transaction manager
    at file-exchange/src/transaction_manager/mod.rs:32

  INFO file_exchange::transaction_manager::staking: unallocate params, allocation_id: 0xe37b9ee6d657ab5700e8a964a8fcc8b39cdefd73
    at file-exchange/src/transaction_manager/staking.rs:142

  DEBUG file_exchange::transaction_manager::staking: estimate gas, estimated_gas: 390965
    at file-exchange/src/transaction_manager/staking.rs:154

  INFO file_exchange: Transaction result, result: Ok((0xe37b9ee6d657ab5700e8a964a8fcc8b39cdefd73, Some(TransactionReceipt { transaction_hash: 0xd5c7c4d3dbd4aa8f845f87f8225aef91e927fe7cd5a1cd02085b0d30a59f4743, transaction_index: 1, block_hash: Some(0xcb46a88b2a37648a38165ca3740248b9a2a41e01f3b56f65f59b33f5cbf00fd0), block_number: Some(5738566), from: 0xe9a1cabd57700b17945fd81feefba82340d9568f, to: Some(0x865365c425f3a593ffe698d9c4e6707d14d51e08), cumulative_gas_used: 345329, gas_used: Some(345329), contract_address: None, logs: [...], status: Some(1), root: None, logs_bloom: ..., transaction_type: Some(2), effective_gas_price: Some(100000000), other: OtherFields { inner: {"gasUsedForL1": String("0x28a70"), "l1BlockNumber": String("0x4d09a3")} } })))
    at file-exchange/src/main.rs:88
```
