# Files Server

The server utilizes HTTP2 with TLS for secure and efficient file transfer. This ensures data integrity and confidentiality while leveraging the performance benefits of HTTP2.

This document provides an overview of the files server, intended for those familiar with blockchain nodes and large datasets.

Jump to [Quick Start](###getting-started)

## Access Control

### Payments 

The server offers

- Meta queries: Server will always respond to queries about their operator info, software versioning, server health, files in service, and prices.
- Free Query Auth Token: Users can obtain a free query auth token for limited access to files. This token allows them to download small files.
- Receipts: Users need to provide TAP receipts in the HTTP header. These receipts serve as proof of payment and grant access to the requested resources.

### Memory access

The server can serve data stored as files or objects

- Local file system: Provided with a directory path, the server can read files from the directory directly by configuring file's relative path to the directory.
- Remote Object storage:  Provided with an S3 bucket configuration, the server can find and read objects from the bucket by configuring object's name and prefix relative to the bucket.

## Server Management

The server includes an admin endpoint for on-the-fly management. This endpoint allows administrators to perform various tasks such as:

- Adding and removing files
- Monitoring server status
- (TODO) maintenance operations such as modifying auth token and pricing.

## Technical Stack
The server utilizes a combination of open-source technologies for optimal performance and security. The core components include:


- Storage system: Server takes a remote object storage paths or a local directory path to read and serve objects/files. 
- Smart Contract: Solidity for secure and transparent server registration and discovery.
- User Interface: CLI to start up the server, HTTP requests for managing files and accessing receipts (TODO: Terminal UI).

## Requirements
- Database: Underlying indexer service framework requires a PostgreSQL database for receipt management.
        - Server management starts with the configuration file, admin for file paths for local access changes the server setting in cache but not stored
        - Later we will utilize PostgreSQL for persisted server management
- `file-exchange` CLI for publishing files or bundles to share with potentital costumers (refer to [Publisher Guide](./publisher_guide.md))


## Installation and Configuration

### Database setup/migrations

Indexer service does _NOT_ run database migrations automatically as it might introduce conflicts with the migrations run by indexer agent. Indexer agent is solely responsible for syncing and migrating the database. If your subgraph indexer-service or indexer-agent have been upgraded to use Scalar-TAP for payments, I recommend using the same database directly (no migrations needed). Otherwise, use a separate database and run the migrations provided as described below.

#### Prerequisite: Install sqlx-cli

Run `cargo install sqlx-cli --no-default-features --features native-tls,postgres` 

Simple option: run general installation that supports all databases supported by SQLx
`cargo install sqlx-cli`

#### Run the migration

Run `sqlx migrate run --source file-service/migrations`

### Getting Started

1. Download either the binary, docker image, or source code from the repository.
2. Referencing `/file-server/template.toml` file, fill in your own configurations.
3. For the initialization bundles, it may be left empty, or you could try publishing a set of files first using the `file-exchange` CLI.
4. Build and run the server.

CLI with configuration file
```
cargo run -p file-service -- --config ./file-server/template.toml
```

(You might need to set an additional envrionmental variable for logs, `RUST_LOG=file-service=debug`)

3. Access services via the additional endpoints:



**Cost and Status API**

Schema is provided at the `server-[]-schema.json`. One can access the graphql playground by navigating to the server's endpoint at `/files-status` or `/files-cost`.

Through curl, an example cost query
```
curl -X POST \ 
        -H 'Content-Type: application/json' \
        --data '{"query": "{costModels(deployments: ["Qm,,,"]){deployment}}"}' \
        http://localhost:5677/files-cost
```

Example status query
```
curl -X POST \
        -H 'Content-Type: application/json' \
        --data '{"query": "{bundles{ipfsHash manifest{fileType description} }}"}' \
        http://localhost:5677/files-status
{"data":{"bundles":[{"ipfsHash":"QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v","manifest":{"fileType":"flatfiles","description":"random flatfiles"}}]}}%    

curl -X POST \
        -H 'Content-Type: application/json' \
        --data '{"query": "{files{totalBytes chunkSize}}"}' \ 
        http://localhost:5677/files-status
{"data":{"files":[{"totalBytes":1052737,"chunkSize":1048576},{"totalBytes":24817953,"chunkSize":1048576},{"totalBytes":26359000,"chunkSize":1048576}]}}%   
```

**Admin API**

Available mutations you can make, in addition to Status queries, are to add and remove bundle(s). If you supplied an admin token, then mutation functions will require the token in the request header.

Curl query will be similar to the above examples, here we provide an example in the GraphQL version
```
mutation{
  addBundles(deployments:["QmeD3dRVV6Gs84TRwiNj3tLt9mBEMVqy3GoWm7WN8oDzGz", "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v"], 
    locations:["./example-file", "./example-file"]){
    ipfsHash
  }
}
```
with configuration 
```
{"authorization": "Bearer admin-token"}
```
(Correspondingly add header `-H 'authorization: Bearer admin-token'` in curl.)


4. (TODO) Register the server endpoint on the smart contract. Currently we assume the service endpoint has been registered with indexer-agent (for subgraphs). 

5. To be compatible with V1 and Scalar TAP, an indexer must maintain an allocation. This means the indexer should use the `wallet` subcommand to create allocations. Refer to [Onchain Guide](onchain_guide.md).

You are open for business!

### Performance and Monitoring

Basic service metrics are hosted at the address configued by `common.server.metrics_host_and_port`, default at "0.0.0.0:7601". Optionally separate metrics are tracked specifically for file service performances at `server.metrics_host_and_port`. The metrics are minimal and please submit feedback for additional specific measurements.

### Security Considerations

The server enforces various security measures to protect user data and system integrity. These measures include:

- Secure communication protocols (HTTPS)
- Access control mechanisms
- Regular security updates

It is crucial to follow best practices for server security and maintain the software updated to mitigate any potential vulnerabilities.

### Support and Community

For further assistance, please consult the following resources:

- Discord channel (to be created)
- Documentation
- GitHub repository

We encourage you to actively participate in the community to share feedback, report issues, and contribute to the project's development.

### Next Steps

This document provides a high-level overview of the server. We encourage you to explore the additional documentation and resources to gain a deeper understanding of the server's capabilities and configuration options.

We are confident that this server will empower you to leverage your technical expertise and extensive file collection to contribute to the decentralized file sharing ecosystem.

