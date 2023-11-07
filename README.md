# Subfile-service

Enable file sharing as a service, aim for a decentralized, efficient, and verifiable market, with scalable, performant, and secure software.

## PoC checklist

### Essential Components
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
    - [ ] later, take a list of files, use File hasher to hash all files and get root hashes 
  - [x] Construct a subfile manifest with metainfo using YAML builder
    - [ ] vectorize
  - [ ] May include a status endpoint for the "canonical" publisher, but recognize the endpoint may change later on
  - [x] Publish subfile to IPFS, receive a IPFS hash for the subfile
- [x] IPFS client
  - [x] Connect to an IPFS gateway
  - [x] Post files
  - [x] Cat files
- [x] YAML parser and builder
  - [x] Deserialize and serialize yaml files
- [ ] Subfile server 
  - [x] Initialize service; for one subfile, take (ipfs_hash, local_path)
    - [x] Take a subfile IPFS hash and get the file using IPFS client
    - [x] Parse yaml file for all the chunk_file hashes using Yaml parser, construct the subfile object 
      - [ ] Take metainfo of chunk_file and search for access by the local_path
      - [ ] Verify the local version satisfy the chunk hashes
    - [ ] Once all verified, add to file to the service availability endpoint
  - [ ] Route `/status` for availability
  - [ ] Route `/subfiles/id/:id` for a subfile using IPFS hash
  - [ ] Route `/health` for general health
  - [ ] Route `/version` for subfile server version
  - [ ] Configure and check free query auth token
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
  - [ ] Start with free service and requiring a free query auth token
    - [ ] then add default cost model, allow updates for pricing per byte
    - [ ] with paid service, validate receipts pricing wrt cost model
  - [ ] Runs TAP agent for receipt management
- [ ] Subfile Client 
  - [ ] Request (ipfs_hash, budget) from the chain after reading the subfile manifest
    - [ ] This may live somewhere else (Gateway?)
      - [ ] Read subfile manifest and construct receipts using budget and chunk sizes
      - [ ] Ping indexer endpoints for pricing and performances, run indexer selection
      - [ ] Construct and send requests (may be parallel) to indexer endpoints 
  - [ ] Wait for the responses (For now, assume that the response chunks correspond with the verifiable chunks)
    - [ ] Keeps track of the downloaded and missing pieces, continually requesting missing pieces until the complete file is obtained
    - [ ] Upon receiving a response, verify the chunk data in the chunk_file
      - [ ] if failed, blacklist the indexer
    - [ ] Once all chunks for a file has been received, verify the file in subfile (should be vacuously true)
  - [ ] Once all file has been received and verified, terminate and notify client



<!-- ### Client

Minimal
- [ ] Use IPFS client to cat the subfile,
- [ ] Parse bytes into a subfile yaml file, fit into a subfile struct, 
- [ ] Grab the chunk_file hashes from subfile.yaml,
- [ ] Use IPFS client to cat the chunk_file,
- [ ] Parse bytes into a chunk_file, fit into a chunk_file struct, 
- [ ] Ping endpoint

Optional
- [ ] Validate IPFS against extra input to make sure it is the target file

### Seeder

Minimal
- [x] Take a file creation arg 
- [ ] Not working: include working tracker info in torrent file ()
- [x] generate a magnet link for the file living at `file_path`
- [x] populate a subfile struct from args
- [x] convert subfile to yaml, containing magnet link and other metadata info
- [x] add subfile.yaml to ipfs using IPFS client
- [x] log out the newly generated ipfs hash of subfile.yaml
- [ ] Not working: Announce to corresponding tracker - should be 
- [ ] Start a torrent peer
- [ ] Start seeding configured subfiles including the ones just created

Optional
- [ ] Whitelist a particular torrent peer
- [ ] Extensive torrent file creation configurations

### Optional: tracker using `torrust-tracker`

- [ ] Ensure tracker availability
- [ ] Configurably private

### CLI Usage

Set RUST_LOG envvar
Provide preferred ipfs gaetway, fallback with thegraph ipfs gateway
Set Log format, fallback with pretty

```
> cargo run -p subfile-cli
Subfile data service - hackathon

Usage: subfile-cli <COMMAND>

Commands:
  leecher  
  seeder   
  tracker  
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Usage: subfile-exchange leecher --ipfs-hash <IPFS_HASH>
Usage: subfile-exchange tracker --server-host <SERVER_HOST> --server-port <SERVER_PORT>
Usage: subfile-exchange seeder --file-path <FILE_PATH> --file-type <FILE_TYPE> --identifier <IDENTIFIER> ...

```

>! Only udp trackers are supported in imdl

## Seeder help
```
> cargo run -p subfile-cli seeder --help
Usage: subfile-cli seeder [OPTIONS] --file-path <FILE_PATH> --file-type <FILE_TYPE> --file-version <FILE_VERSION> --identifier <IDENTIFIER>

Options:
      --file-config <SUBFILE_SEEDS>  A vector of ipfs hashes to the subfiles to support seeding for [env: SUBFILE_SEEDS=]
      --yaml-store <YAML_STORE_DIR>  Path to the directory to store the generated yaml file for subfile [env: YAML_STORE_DIR=] [default:
                                     ./example-file/subfile.yaml]
      --file-path <FILE_PATH>        Path to the file for seeding [env: FILE_PATH=]
      --name <TORRENT_NAME>          Target torrent name [env: TORRENT_NAME=]
      --file-type <FILE_TYPE>        Type of the file (e.g., sql_snapshot, flatfiles) [env: FILE_TYPE=]
      --file-version <FILE_VERSION>  Subfile Versioning [env: FILE_VERSION=]
      --identifier <IDENTIFIER>      Identifier of the file given its type [env: IDENTIFIER=]
      --start-block <START_BLOCK>    Start block for flatfiles [env: START_BLOCK=]
      --end-block <END_BLOCK>        End block for sql snapshot or flatfiles [env: END_BLOCK=]
      --trackers <TRACKER_URL>       Annouce torrent file to at the tracker URL. [env: TRACKER_URL=]
  -h, --help                         Print help
```

Example
```
> cargo run -p subfile-cli seeder \
  --file-name graph-node-simple.sql \
  --file-path ./data-files/graph-node-simple.sql \
  --file-type sql_snapshot \
  --identifier Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX \
  --file-version "0.0.1-a" \
  --trackers udp://tracker.opentrackr.org:1337/announce,udp://opentracker.i2p.rocks:6969/announce

  INFO subfile_cli: Running cli, cli: Cli { role: Seeder(Seeder { file_config: [], yaml_store: "./example-file/subfile.yaml", file_path: "./example-file/graph-node-simple.sql", name: None, file_type: "sql_snapshot", file_version: "0.0.1", identifier: "Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX", start_block: None, end_block: None, trackers: ["https://tracker1.520.jp:443"] }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-cli/src/main.rs:20

  INFO subfile_cli: Seeder request, seeder: Seeder { file_config: [], yaml_store: "./example-file/subfile.yaml", file_path: "./example-file/graph-node-simple.sql", name: None, file_type: "sql_snapshot", file_version: "0.0.1", identifier: "Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX", start_block: None, end_block: None, trackers: ["https://tracker1.520.jp:443"] }
    at subfile-cli/src/main.rs:49

  INFO subfile_cli::torrent::create: Generated Torrent file
    at subfile-cli/src/torrent/create.rs:154

  INFO subfile_cli::torrent::create: Generated Torrent file, summary: TorrentSummary { infohash: Infohash { inner: Sha1Digest { bytes: [202, 49, 59, 174, 101, 101, 142, 1, 7, 214, 163, 10, 72, 202, 75, 22, 102, 37, 102, 54] } }, metainfo: Metainfo { announce: Some("https://tracker1.520.jp:443"), announce_list: None, comment: None, created_by: None, creation_date: None, encoding: Some("UTF-8"), info: Info ...}}
    at subfile-cli/src/torrent/create.rs:157

  INFO subfile_cli::torrent::create: Magnet Link, link: MagnetLink { infohash: Infohash { inner: Sha1Digest { bytes: [202, 49, 59, 174, 101, 101, 142, 1, 7, 214, 163, 10, 72, 202, 75, 22, 102, 37, 102, 54] } }, name: Some("filename.sql"), peers: [], trackers: [Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("tracker1.520.jp")), port: None, path: "/", query: None, fragment: None }], indices: {} }
    at subfile-cli/src/torrent/create.rs:165

  INFO subfile_cli::seeder: Added yaml file to IPFS, added: AddResponse { name: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT", hash: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT", size: "318" }, client: IpfsClient { base: https://ipfs.network.thegraph.com/, client: Client { accepts: Accepts, proxies: [Proxy(System({}), None)], referer: true, default_headers: {"accept": "*/*"} } }
    at subfile-cli/src/seeder.rs:47

  INFO subfile_cli: Completed seed, result: AddResponse { name: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT", hash: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT", size: "318" }
    at subfile-cli/src/main.rs:58

```

=> file added at `https://ipfs.network.thegraph.com/api/v0/cat?arg=[^hash]`




## Leecher 
```
Usage: subfile-cli leecher --ipfs-hash <IPFS_HASH>

Options:
      --ipfs-hash <IPFS_HASH>  IPFS hash for the target subfile.yaml [env: IPFS_HASH=]
  -h, --help                   Print help
```

Example

```
cargo run -p subfile-cli leecher --ipfs-hash QmfE69Xe143tbwhhjAzSpKHDvrtTdHZAKH6QYNf92pJd3Q

INFO subfile_cli: Running cli, cli: Cli { role: Leecher(Leecher { ipfs_hash: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT" }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-cli/src/main.rs:20

INFO subfile_cli: Leecher request, leecher: Leecher { ipfs_hash: "QmbYPsAsXomUcFrVNyx1sL3kc5ELJhSi96QZ3VQT1sD5NT" }
  at subfile-cli/src/main.rs:24

INFO subfile_cli::leecher: Got yaml file content
  at subfile-cli/src/leecher.rs:25

TRACE subfile_cli::leecher: Parse yaml value into a subfile, value: Mapping {"magnet_link": String("magnet:?xt=urn:btih:ca313bae65658e0107d6a30a48ca4b1666256636&dn=filename.sql&tr=https://tracker1.520.jp/"), "file_type": String("sql_snapshot"), "version": String("0.0.1"), "identifier": String("Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX"), "trackers": Sequence [String("https://tracker1.520.jp:443")], "block_range": Mapping {"start_block": Null, "end_block": Null}}
  at subfile-cli/src/leecher.rs:33

TRACE subfile_cli::leecher: Grabbed subfile, magnet_link: "magnet:?xt=urn:btih:ca313bae65658e0107d6a30a48ca4b1666256636&dn=filename.sql&tr=https://tracker1.520.jp/"
  at subfile-cli/src/leecher.rs:46

INFO subfile_cli: Completed leech, result: Subfile { magnet_link: "magnet:?xt=urn:btih:ca313bae65658e0107d6a30a48ca4b1666256636&dn=filename.sql&tr=https://tracker1.520.jp/", file_type: "sql_snapshot", version: "0.0.1", identifier: "Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX", trackers: ["https://tracker1.520.jp:443"], block_range: BlockRange { start_block: None, end_block: None } }
    at subfile-cli/src/main.rs:41
``` -->






## Strike to be more like subgraph.yaml

Minimal 
- [ ] camel case
- [ ] `version` -> `specVersion`
- [ ] description
- [ ] dataSources array
  - [ ] kind (subgraph deployment snapshot)
  - [ ] name (subgraph name)
  - [ ] network
  - [ ] block range (subgraph indexing network)
  - [ ] source:
    - [ ] address: deployment hash
    - [ ] abi: subgraph graphQL schema
    - [ ] snapshot_block

Optional
- [ ] repository
- [ ] schema: file: ./schema.graphql
- [ ] mapping: (directory composition of files)

[subfile_manifest.md]
