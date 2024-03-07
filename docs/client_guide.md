# File Sharing Client

Tired of data drudgery? Imagine a world where you skip the tedious task of indexing data and start serving queries in a flash! That's the magic of our file sharing network. Instead of spending hours to months catching up to the head of a chain, you can tap into a vast pool of indexed information, ready and waiting for your queries. 

This document provides an overview of the file sharing client.

## Functionality

The client facilitates data retrieval from FHS. The client implements:

- File retrieval: Download entire files or specific chunks at a time.
- Payment: Pay for file access using tokens deposited in an Escrow account on-chain.
- Free access: Utilize a free query auth token for limited access to particular servers.
- Indexer selection: Choose from multiple indexer endpoints; Currently just for availability, later add optimization for performance and redundancy.

## Minimal Trust Requirement

To minimize trust requirements, the client employs a chunk-based payment system. Instead of paying for the entire file upfront, users can pay for individual chunks as they download and verify them. This ensures transparency and reduces the risk of losing funds due to server downtime or malicious actors.

## CLI Usage

The client operates through a command-line interface (CLI) for simplicity and ease of use. Client would need to determine the Bundle that contains the dataset they desire. This may mean looking at Bundle manifests or in the future a tool that matches manifests by the provided critieria. 

After determining the Bundle CID, client should supply a local path for writing the Bundle corresponding files, a wallet for payments or a free query auth token, and a list of indexer endpoints (this should be handled by gateway or a scraping client).

### CLI example

Download into local file system

```
$ file-exchange downloader \
   --ipfs-hash QmHash \
   --indexer-endpoints http://localhost:5678,http://localhost:5677 \
   --free-query-auth-token 'Bearer auth_token' \
   --mnemonic "seed phrase" \
   --verifier 0xfC24cE7a4428A6B89B52645243662A02BA734ECF \
   --provider "arbitrum-sepolia-rpc-endpoint" \
   --network-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-arbitrum-sepolia \
   --escrow-subgraph https://api.thegraph.com/subgraphs/name/graphprotocol/scalar-tap-arbitrum-sepolia \
   --provider-concurrency 2 \
   --max-auto-deposit 500 \
   local-files --output-dir "../example-download"
```

Download into remote object storage bucket

```
$ file-exchange downloader \
   --ipfs-hash QmHash \
   --indexer-endpoints http://localhost:5678,http://localhost:5677 \
   --free-query-auth-token 'Bearer auth_token' \
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

### Requirements

To use the client effectively, you will need:

- Bundle Manifest CID: The CID of the Bundle Manifest you want to download.
- Local Path: A directory where the downloaded file will be stored. (Later will be a generic storage path, enabling cloud storage access)
- Wallet: A blockchain wallet containing tokens for escrow payments.
- Indexer Endpoints: A list of available server addresses.
- (Optional) Free Query Auth Token: For limited access to small files.

### Getting Started

1. Download and install the source code.
2. Gather configurations: Identify the CID of the desired Bundle, registered indexer endpoints, a local path for storing the downloaded files, private key (or mnemonics) of a wallet valid for Escrow payments, (optional) Obtain a free query auth token for limited access, the preference to concurrent providers for downloading.
3. Use the CLI commands to download files.
4. Before downloading, the client will check the status and price of the providers. If the download can be achived by availablility and price at the time of initiation, then download will proceed. If there is no availability, the client will suggest alternative bundles that overlaps with the target bundle and the corresponding providers. If there is not enough balance, the client will suggest Escrow top-up amounts for the Escrow accounts. With a configured on-chain deposit, the downloader might send GraphToken approval transaction to approve Escrow spending and deposit required amounts to the providers.  

Enjoy seamless access to a vast world of data!

### Security Considerations

The client prioritizes user safety and security. It employs secure communication protocols and wallet management practices. However, users should always be mindful of potential risks:

- Choosing manifests: Verify correctness of the Bundle before initiating file requests.
- Securing wallet: Implement strong key protection and other security measures for the wallet.
- Staying informed: Updated on the latest security threats, invalid manifests, and updates for the client software.

### Join the Community

To learn more, share experiences, and contribute to the network's growth.

- Discord channel to be created
- Documentation: Explore detailed guides and technical information.
- GitHub Repository: Contribute to the client's development and propose improvements.
