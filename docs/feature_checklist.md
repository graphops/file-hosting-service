
## Feature checklist

- [x] File hasher
  - [x] use sha2-256 as it is more commonly used, faster than sha3-256, both no known hacks (should be easy to switch)
  - [x] Takes a file path and read
  - [x] Chunk file to a certain size - currently using a constant of 1MB
  - [X] Hash each chunk as leaves (nodes)
  - [x] Produce a merkle tree
  - [x] construct and write a chunk_file.yaml (root, nodes)
  - [x] Unit tests: same file same hash, different file different hash, big temp file same/modified
  - [x] last chunk lengths, 
  - [ ] Analyze merkle tree vs hash list
  - [ ] memory usage for hashing (profiling to O(n) where n is the size of the file)
- [ ] Subfile builder / publisher - CLI
  - [x] Take a file, use File hasher to get the chunk_file, publish chunk_file to IPFS
    - [x] later, take a list of files, use File hasher to hash all files and get root hashes 
  - [x] Construct a subfile manifest with metainfo using YAML builder
    - [x] vectorize
  - [ ] May include a status endpoint for the "canonical" publisher, but recognize the endpoint may change later on
  - [x] Publish subfile to IPFS, receive a IPFS hash for the subfile
- [x] IPFS client
  - [x] Connect to an IPFS gateway
  - [x] Post files
  - [x] Cat files
- [x] YAML parser and builder
  - [x] Deserialize and serialize yaml files
- [ ] Subfile server 
  - [x] require operator mnemonic
  - [x] Initialize service; for one subfile, take (ipfs_hash, local_path)
    - [x] Take a subfile IPFS hash and get the file using IPFS client
    - [x] Parse yaml file for all the chunk_file hashes using Yaml parser, construct the subfile object 
      - [x] Take metainfo of chunk_file and search for access by the local_path
      - [ ] Verify local file against the chunk hashes
    - [x] vectorize service for multiple subfiles
    - [ ] Once all verified, add to file to the service availability endpoint
  - [x] Route `/` for "Ready to roll!"
  - [x] Route `/operator` for operator info
  - [x] Route `/status` for availability
    - [ ] verification for availability
  - [x] Route `/subfiles/id/:id` for a subfile using IPFS hash with range requests
  - [x] Route `/health` for general health
  - [x] Route `/version` for subfile server version
  - [x] Configure and check free query auth token
  - [ ] Server Certificate 
  - [ ] Upon receiving a service request (ipfs_hash, range, receipt)
    - [x] start off with request as (ipfs_hash, range)
    - [x] Check if ipfs_hash is available
    - [x] Check if range is valid against the subfile and the specific chunk_file
    - [ ] Valid and store receipt
    - [x] Read in the requested chunk
      - [ ] Add tests
    - [x] Construct response and respond
      - [ ] determine if streaming is necessary
  - [x] Start with free service and requiring a free query auth token
    - [ ] then add default cost model, allow updates for pricing per byte
    - [ ] with paid service, validate receipts pricing wrt cost model
  - [ ] Runs TAP agent for receipt management
- [ ] Subfile Client 
  - [ ] Take private key/mneomic for wallet connections
  - [x] Request using ipfs_hash
    - [ ] take budget for the overall subfile
      - [ ] construct receipts using budget and chunk sizes
      - [ ] add receipt to request
    - [x] add free_token to request
      - [ ] match token with indexer-urls
    - [ ] This may live somewhere else (Gateway?)
      - [x] Read subfile manifest
    - [x] Ping indexer endpoints data availability
    - [ ] Ping indexer endpoints for pricing and performances, run indexer selection
      - [x] Use random endpoints
    - [x] Construct and send requests to indexer endpoints 
      - [ ] Parallelize requests
      - [ ] Multiple connections (HTTPS over HTTP2)
  - [x] Wait for the responses (For now, assume that the response chunks correspond with the verifiable chunks)
    - [x] Keeps track of the downloaded and missing pieces, continually requesting missing pieces until the complete file is obtained
    - [x] Upon receiving a response, verify the chunk data in the chunk_file
      - [ ] if failed, blacklist the indexer
    - [ ] Once all chunks for a file has been received, verify the file in subfile (should be vacuously true)
  - [x] Once all file has been received and verified, terminate
