# File Sharing Server

You hold the key to unlocking the network's true potential. By sharing your meticulously indexed data, you become the architect of information accessibility. Imagine, your contribution fueling a vibrant ecosystem of knowledge, where fellow indexers can build upon your work, unleashing a torrent of information to the world. In return, your generosity is rewarded with precious tokens, a testament to the invaluable role you play in this decentralized revolution. Become the data hero we need, and together, let us build a brighter future fueled by open access and boundless knowledge!

This document provides an overview of the P2P file sharing server, intended for those familiar with blockchain nodes and large datasets.

Jump to [Quick Start](###getting-started)

## File Transfer Protocol

The server utilizes HTTP2 over HTTPS for secure and efficient file transfer. This ensures data integrity and confidentiality while leveraging the performance benefits of HTTP2.


## Access Control

The server offers

- Free queries: Server will always respond to queries about their operator info, software versioning, server health, and file availability.
- Free Query Auth Token: Users can obtain a free query auth token for limited access to files. This token allows them to download small files.
- Receipts: Users need to provide TAP receipts in the HTTP header. These receipts serve as proof of payment and grant access to the requested resources.

## Server Management

The server includes an admin endpoint for on-the-fly management. This endpoint allows administrators to perform various tasks such as:

- Adding and removing files
- Monitoring server status
- (TODO) maintenance operations such as modifying auth token and pricing model.

## Technical Stack
The server utilizes a combination of open-source technologies for optimal performance and security. The core components include:

- Backend: Rust for robust and efficient server-side processing.
- Database: (Current: In-memory for server management, file paths for local access). 
- Database: (TODO: PostgreSQL for persisted server management, generic storage paths to allow cloud/object storage). 
- Smart Contract: Solidity for secure and transparent server registration and discovery.
- User Interface: CLI to start up the server, HTTP requests for managing files and accessing receipts (TODO: Terminal UI).

## System Requirements
- Operating System: Linux/MacOS
- RAM: __
- Disk Space: ___
- CPU: ___
- Rust: 1.73.0
- Docker (optional, TODO)

## Installation and Configuration

### Getting Started

1. Download the source code from the repository.
2. Build and run the server.

CLI example
```
✗ cargo run -p subfile-exchange server \
  --host 0.0.0.0 \
  --port 5678 \
  --mnemonic "seed phrase" \
  --admin-auth-token "imadmin" \
  --free-query-auth-token "imafriend" \
  --subfiles "QmHash00:./example-file/,QmHash01:SUBFILE_PATH"
```
Run `cargo run -p subfile-exchange --help` for more configurations and the corresponding ENV variable names.

3. Access the server via the **admin** endpoint.

HTTP request example to get, add, and remove subfile services
```
✗ curl http://localhost:5678/admin -X POST \
  -H "Content-Type: application/json" \
  -H "AUTHORIZATION: Bearer imadmin" \
 --data '{"method":"add_subfile","params":["QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ:./example-file"],"id":1,"jsonrpc":"2.0"}' 
Subfile(s) added successfully%      

✗ curl http://localhost:5678/admin -X POST \
  -H "Content-Type: application/json" \
  -H "AUTHORIZATION: Bearer imadmin" \
 --data '{"method":"get_subfiles","id":1,"jsonrpc":"2.0"}'
[{
  "ipfs_hash":"QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ","subfile":{"chunk_files":[{"chunk_hashes":["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=","TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="],"chunk_size":1048576,"file_name":"example-create-17686085.dbin","total_bytes":1052737},{"chunk_hashes":["/5jJskCMgWAZIZHWBWcwnaLP8Ax4sOzCq6d9+k2ouE8=",...],"chunk_size":1048576,"file_name":"0017234500.dbin.zst","total_bytes":24817953},...],
"ipfs_hash":"QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ","local_path":"./example-file","manifest":{"block_range":{"end_block":null,"start_block":null},"chain_id":"0","description":"random flatfiles","file_type":"flatfiles","files":[{"hash":"QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv","name":"example-create-17686085.dbin"}, ...],"spec_version":"0.0.0"}}}, ...]%                            

✗ curl http://localhost:5678/admin -X POST \
  -H "Content-Type: application/json" \
  -H "AUTHORIZATION: Bearer imadmin" \
 --data '{"method":"remove_subfile","params":["QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ"],"id":1,"jsonrpc":"2.0"}' 
Subfile(s) removed successfully
```

4. (TODO) Register the server endpoint on the smart contract. Currently we assume the service endpoint has been registered with indexer-agent (for subgraphs). 

5. To be compatible with V1 and Scalar TAP, an indexer must maintain an allocation. This means the indexer should use the `wallet` subcommand to create allocations. Refer to [Onchain Guide](onchain_guide.md).

You are open for business!

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

