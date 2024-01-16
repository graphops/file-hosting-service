# File Hosting Service

## Introduction 

File Hosting Service (FHS) is a marketplace for sharing file data and is part of The Graph Network's [World of Data Services](https://forum.thegraph.com/t/gip-0042-a-world-of-data-services/3761).

FHS is a decentralized, peer-to-peer data sharing platform designed for efficient and trust-minimised file sharing that is payments-enabled. It leverages a combination of technologies including hash commitments on IPFS for file discovery and verification, chunked data transfer and micropayments reducing trust requirements between clients and servers, and secure and efficient data transfers via HTTP2. The system is built with scalability, performance, integrity, and security in mind, aiming to create a robust market for file sharing.

## Target Audience

This documentation is tailored for individuals who have a basic understanding of decentralized technologies, peer-to-peer networks, and cryptographic principles. Whether you are an indexer running various blockchain nodes looking for sharing and verifying your data, an indexer looking to launch service for a new chain, or simply a user interested in the world of decentralized file sharing, this guide aims to provide you with a clear and comprehensive understanding of how File Service operates.

## Features

- Decentralized File Sharing: FHS uses direct connections for file transfers, eliminating central points of failure.
- IPFS Integration: Employ IPFS for efficient and reliable file discovery and content verification.
- SHA2-256 Hashing: Ensure data integrity through robust and incremental cryptographic hashing.
- HTTP2 and TLS: Leverage the latest web protocols for secure and efficient data transfer.

**To be supported:**
- Micropayments Support: Implement a system of micropayments to facilitate fair compensation and reduce trust requirements.
- Scalability and Performance: Designed with a focus on handling large volumes of data and high user traffic.
- User-Friendly Interface: Intuitive design for easy navigation and operation.

More details can be found in [Feature Checklist](docs/feature_checklist.md)

## Upgrading

The project will follow conventional semantic versioning specified [here](https://semver.org/). Server will expose an endpoint for package versioning to ensure correct versions are used during exchanges. 

## Background Resources

You may learn background information on various components of the exchange

1. **Cryptography**: [SHA2-256 Generic guide](https://blog.boot.dev/cryptography/how-sha-2-works-step-by-step-sha-256/), [Hashed Data Structure slides](https://zoo.cs.yale.edu/classes/cs467/2020f/lectures/ln16.pdf)

2. **Networking**: [HTTPS](https://crypto.stanford.edu/cs142/lectures/http.html) with [SSL/TLS](https://cs249i.stanford.edu/lectures/Secure%20Internet%20Protocols.pdf).

3. **Specifications**: [IPFS](https://docs-ipfs-tech.ipns.dweb.link/) file storage, retrieval, and content addressing.

4. **Blockchain**: [World of data services](https://forum.thegraph.com/t/gip-0042-a-world-of-data-services/3761), [flatfiles for Ethereum](https://github.com/streamingfast/firehose-ethereum), [use case](https://eips.ethereum.org/EIPS/eip-4444).

## Documentation

#### [Architecture](docs/architecture.md)

#### [Entity Definitions](docs/manifest.md)

#### [Contracts](docs/contracts.md)

### Quickstarts and Configuring

#### [Server Guide](docs/server_guide.md)

#### [Client Guide](docs/client_guide.md)

#### [Publisher Guide](docs/publisher_guide.md)

#### [On-Chain Guide](docs/onchain_guide.md)

## Contributing

We welcome and appreciate your contributions! Please see the [Contributor Guide](/CONTRIBUTING.md), [Code Of Conduct](/CODE_OF_CONDUCT.md) and [Security Notes](/SECURITY.md) for this repository.
