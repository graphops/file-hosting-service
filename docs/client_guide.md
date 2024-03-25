# File Downloader

This document provides an overview of the file download client.

## Functionality

The client facilitates data retrieval from FHS, handles several functions such as

- File retrieval: Download files from file servers and store in local filesystem or remote object storage.
- Payment: Pay for file access using tokens deposited in on-chain Escrow accounts; alternatively, they can acquire free query auth token for accessing particular servers.
- Indexer selection: Choose from multiple indexer endpoints based on availability and price, with configurable redundancy.

## Minimal Trust Requirement

To minimize trust requirements, the client employs a chunk-based payment system. Instead of paying for the entire file upfront, users can pay for individual chunks as they download and verify them. This ensures transparency and reduces the risk of losing funds due to server downtime or malicious actors. For detailed description, refer to documentation on [manifest](./manifest.md)

## Limitations

1. Consumers must gather a set of file service endpoints. At the current stage of the protocol, they will gather the list off-chain, either through private exchanges, forum posts, or public channels. Automatic on-chain discovery of available file services can be accomplished after Horizon's deployment of data service contracts. 

2. Consumers are responsible for determining which bundle contains files/data they desire. They will find available bundles from a set of indexer endpoints, and read through bundle manifests for descriptions of the files. This requires the consumer to place trust to particular Bundle manifests. Critically, consumers must understand that the tool guarantees that once a manifest has been picked, the transferred and stored data can be verified against hashes contained in the manifest; this tool **does not** guarantee the correctness of manifest descriptions. In the future, we can provide a separate tool or service that check for or challenge manifest description correctness. 

### Requirements

To use the client effectively, you will need:

- Bundle Manifest CID: The CID of the Bundle Manifest you want to download.
- Indexer Endpoints: A list of available server addresses.
- Storage options
   - Local Path: A directory where the downloaded file will be stored. (Default: "./example-download")
   - Remote Path: S3 bucket configurations including endpoint, access key id, secret key, bucket name, and region
- Payment options
   - Wallet: A blockchain wallet containing tokens for escrow payments.
   - Free Query Auth Token: For limited access to a particular server.

## Usage

The client operates through a command-line interface (CLI) for simplicity and ease of use. 

After gathering a list of indexer endpoints and determining the Bundle CID (`ipfs-hash`), client should also supply a local or remote storage path for storing the downloads. 

If the client provides a free query auth token, the download will use the free query flow, otherwise, the downloader requires payment configurations, which includes a wallet mnemonic, a Escrow verifier, a Eth provider, and optionally a maximum automatic deposit amount.  

### Quick Start CLI example

Download into local file system with free query auth token

```
$ file-exchange downloader \
   --ipfs-hash QmHash \
   --indexer-endpoints http://localhost:5678,http://localhost:5677 \
   --free-query-auth-token 'Bearer auth_token' \
   --network-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-arbitrum-sepolia \
   --escrow-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/scalar-tap-arbitrum-sepolia \
   --provider-concurrency 2 \
   --progress-file "../example-download" \
   local-files --main-dir "../example-file"
```

Download into remote object storage bucket with paid query flow

```
$ file-exchange downloader \
   --ipfs-hash QmHash \
   --indexer-endpoints http://localhost:5678,http://localhost:5677 \
   --mnemonic "seed phrase" \
   --verifier 0xfC24cE7a4428A6B89B52645243662A02BA734ECF \
   --provider "arbitrum-sepolia-rpc-endpoint" \
   --network-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-arbitrum-sepolia \
   --escrow-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/scalar-tap-arbitrum-sepolia \
   --provider-concurrency 2 \
   --max-auto-deposit 500 \
   object-storage --region ams3 \
   --bucket "contain-texture-dragon" \
   --access-key-id "DO000000000000000000" \
   --secret-key "secretttttttttttt" \
   --endpoint "https://ams3.digitaloceanspaces.com"
```

### Getting Started

1. You can use the provided binaries, docker image, or download and install the source code.
2. Gather configurations as described in the above Requirements section. For detailed CLI instructions, run `file-exchange donwloader --help`.
3. Use the CLI commands to download files.

Before downloading, the client will check the status and price of the providers. If the download can be achived by availablility and price at the time of initiation, then download will proceed. 
- If there is no availability, the client will suggest alternative bundles that overlaps with the target bundle and the corresponding providers. 
- If there is not enough balance in the escrow account, the client will suggest Escrow top-up amounts for the Escrow accounts. With a configured on-chain deposit, the downloader might send GraphToken approval transaction to approve Escrow spending and then deposit required amounts to the providers.  

4. Depending on the log setting, there will be logs on the download progress.

### Security Considerations

The client prioritizes user safety and security. It employs secure communication protocols and wallet management practices. However, users should always be mindful of potential risks:

- Choosing manifests: Verify correctness of the Bundle before initiating file requests.
- Securing wallet: Employ strong key protection and other security measures for the wallet.
- Staying informed: Updated on the latest security threats, invalid manifests, and updates for the client software.

### Join the Community

To learn more, share experiences, and contribute to the network's growth.

- Discord channel (to be created)
- Documentation: Explore detailed guides and technical information.
- GitHub Repository: Contribute to the client's development and propose improvements.
