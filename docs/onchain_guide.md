# On-Chain Subcommand Documentation

The subcommand `wallet` is designed to facilitate various on-chain actions through a command-line interface. This guide focuses on how users can perform wallet transactions on the blockchain.

## Configuration

Before starting, ensure you have set up the following:

- **Mnemonic**: A secret phrase (mnemonic) for accessing your wallet.
- **Provider URL**: The endpoint URL of your blockchain provider.

## Usage

To use the Wallet CLI, the following subcommands and options are available:

### Global Options

- `--mnemonic KEY`: Sets the mnemonic for the wallet. This is required for any wallet operations.
- `--provider provider_url`: The blockchain provider endpoint URL.

### Subcommands

- `allocate`: Allocate funds for a specific purpose.
- `unallocate`: Revoke previously allocated funds.

#### Allocate

To open allocation towards a deployment, provide the deployment IPFS hash, the token amount, the current epoch number (should later be resolved automaically), and fill in the `allocate` subcommand with the necessary arguments:

```shell
✗ subfile-exchange wallet \
    --mnemonic <mnemonic> \
    --provider <provider_url> \
    allocate \
        --tokens <tokens> \
        --deployment-ipfs <deployment_ipfs> \
        --epoch <epoch>
```

#### Unallocate

>To be implemented

### Examples

**Allocating Funds:**

```shell
✗ cargo run -p subfile-exchange wallet \
    --mnemonic "mnemonic phrase" \
    --provider "http://localhost:8545" \
    allocate \
        --tokens 100 \
        --deployment-ipfs QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v \
        --epoch 100
```

- Replace placeholders like `"your mnemonic"`, `"http://localhost:8545"`, `"0x123..."`, `"QmHash"`, etc., with actual values.
- Ensure that the mnemonic and provider URL are kept secure and private.

With RUST_LOG turned on, you can expect the following logs upon success
```
  2024-01-08T18:17:34.123941Z  INFO subfile_exchange::transaction_manager: Initialize transaction manager
    at subfile-exchange/src/transaction_manager/mod.rs:32

  2024-01-08T18:17:34.650044Z  INFO subfile_exchange::transaction_manager::staking: allocate params, dep_bytes: [241, 64, 71, 78, 218, 63, 159, 91, 130, 173, 178, 168, 30, 254, 183, 20, 225, 131, 35, 230, 52, 85, 74, 196, 40, 255, 173, 61, 144, 126, 223, 33], tokens: Some(Allocate(AllocateArgs { tokens: 256, deployment_ipfs: "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v", epoch: 101 })), allocation_id: 0x75e11e0f2319913c28d0b1916b4b1d9a1ac3977b, metadata: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], proof: Bytes(0xac1b36d68ea95ebe9f9793850cf083a031d40806121dc2dac525423c50611d18053f195627f6bffe036b9325b4dfd273959457b5d3f1c1b53095c096182756bb1b)
    at subfile-exchange/src/transaction_manager/staking.rs:67

  2024-01-08T18:17:34.765769Z DEBUG subfile_exchange::transaction_manager::staking: estimate gas, estimated_gas: 379109
    at subfile-exchange/src/transaction_manager/staking.rs:82

  2024-01-08T18:17:42.224872Z  INFO subfile_exchange: Allocation transaction finished, allocation_id: 0x75e11e0f2319913c28d0b1916b4b1d9a1ac3977b, tx_receipt: Some(TransactionReceipt { transaction_hash: 0x835b790326abf1555545920265e54d5bfbaba150aef31820529736e6727c7a0a, ... })
    at subfile-exchange/src/main.rs:75
```