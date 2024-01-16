## Architecture and Components

### Generic Diagram

![Diagram](./fhs.png)

### Key Components

1. **File Hasher** ensures data integrity. It uses the SHA2-256 hashing algorithm to process files. The hasher chunks files into manageable sizes (currently 1MB), hashes each chunk, and then organizes these hashes into a Merkle tree structure (Currently we are using an ordered list, but should be relatively simple to update to use the tree structure as it optimizes the verification process ($O(n)$ versus $O(log(n)$ for a single chunk verification where $n$ is the number of chunks), but require 2x memory usage for the hash data structure).

2. **Manifest Publisher** is responsible for preparing and publishing files onto the network. It takes files, processes them through the File Hasher to generate a file_manifest.yaml containing chunk hashes, and then publishes this data to IPFS. The Manifest Builder/Publisher also constructs a file/bundle manifest, which contains metadata and other relevant information about the files.

3. **IPFS Client** connects to the IPFS network as it is used for posting files to the network and retrieving them. IPFS plays a crucial role in discovering and verifying files, as it allows for content-addressable storage.

4. **File Server** requires an operator mnemonic for initialization and handles various tasks such as retrieving files from IPFS, managing file services, verifying file integrity against chunk hashes, and managing API endpoints. The server also implements routes for various functionalities like health checks, version information, and file availability.

5. **File Downloader** is used to request and receive files. It handles the construction of requests, including the addition of authentication tokens and, in future iterations, will manage budgeting for file downloads. The client is responsible for ensuring that the received files are complete and authentic by verifying each chunk against the hashes provided by the File Hasher.
