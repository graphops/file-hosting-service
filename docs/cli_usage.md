

### Publisher 

| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `READ_DIR`             | `--read-dir`             | Read directory for the files to publish               |
| `SUBFILE_NAME`             | `--subfile-name`             | Give a name to the subfile (Q: removable as it can cause unnecessary change in subfile hash)               |
| `FILE_TYPE`             | `--file-type`             | flatfile, snapshot, ...               |
| `FILE_VERSION`             | `--file-version`             | Subfile version               |
| `FILE_NAMES`             | `--file-names`             | name of the files to include in the package               |
| `IDENTIFIER`             | `--identifier`             | Optional, Useful for deployment specific files                |
| `PUBLISHER_URL`             | `--publisher-url`             | Optional, include in subfile manifest for self advertisement               |
| `DESCRIPTION`             | `--description`             | Descibe the subfile content               |
| `SUBFILE_SERVICE_ETHEREUM_NETWORK`            | `--chain-id`            | mainnet: `1`, goerli: `5`, arbitrum-one: `42161`, sepolia: `58008`                              |

### To add
| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `SUBFILE_SERVICE_MNEMONIC`                    | `--mnemonic`                    | Ethereum mnemonic for connecting to a wallet for publishing on-chain           |

Consider access such as to postgres or files requiring authentication


```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange publisher \
  --read-dir ./example-file/ \
  --subfile-name "blah" \
  --file-names example0017686312.dbin,example-create-17686085.dbin \
  --file-type nothing \
  --file-version 0.0.0 \
  --identifier example0017686312 \
  --publisher-url "http://localhost:5678" 

     Running `target/debug/subfile-exchange publisher --read-dir ./example-file/ --subfile-name blah --file-names example0017686312.dbin,example-create-17686085.dbin --file-type nothing --file-version 0.0.0 --identifier example0017686312 --publisher-url 'http://localhost:5678'`
  2023-11-13T20:54:00.508237Z  INFO subfile_cli: Running cli, cli: Cli { role: Publisher(PublisherArgs { yaml_store: "./example-file/subfile.yaml", read_dir: "./example-file/", subfile_name: "blah", file_names: ["example0017686312.dbin", "example-create-17686085.dbin"], file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-exchange/src/main.rs:16

  2023-11-13T20:54:00.521591Z  INFO subfile_cli: Publisher request, config: PublisherArgs { yaml_store: "./example-file/subfile.yaml", read_dir: "./example-file/", subfile_name: "blah", file_names: ["example0017686312.dbin", "example-create-17686085.dbin"], file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }
    at subfile-exchange/src/main.rs:43

  2023-11-13T20:54:00.521793Z  INFO crate::publisher: hash_and_publish_files, file_names: ["example0017686312.dbin", "example-create-17686085.dbin"]
    at subfile-exchange/src/publisher.rs:65

  2023-11-13T20:54:00.521916Z  INFO crate::file_hasher: write_chunk_file, read_dir: "./example-file/", file_name: "example0017686312.dbin"
    at subfile-exchange/src/file_hasher.rs:115

  2023-11-13T20:54:00.523842Z DEBUG crate::file_hasher: Chunked file, file: "./example-file//example0017686312.dbin", total_bytes: 1508787, num_chunks: 2
    at subfile-exchange/src/file_hasher.rs:51

  2023-11-13T20:54:00.576871Z DEBUG crate::file_hasher: Chunk hash, hash: [202, 227, 94, 140, 125, 254, 145, 68, 217, 119, 24, 19, 178, 40, 120, 144, 232, 4, 19, 190, 183, 234, 100, 36, 76, 83, 65, 117, 96, 23, 26, 146]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:54:00.576961Z DEBUG crate::file_hasher: Chunk hash, hash_str: "yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:54:00.600089Z DEBUG crate::file_hasher: Chunk hash, hash: [191, 27, 225, 27, 22, 159, 177, 171, 252, 216, 173, 16, 107, 156, 16, 160, 148, 174, 246, 237, 228, 234, 254, 114, 77, 246, 129, 45, 195, 205, 203, 231]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:54:00.600109Z DEBUG crate::file_hasher: Chunk hash, hash_str: "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:54:00.600226Z TRACE crate::file_hasher: Created chunk file, file: ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }
    at subfile-exchange/src/file_hasher.rs:123

  2023-11-13T20:54:01.523414Z DEBUG crate::publisher: Added yaml file to IPFS, added: AddResponse { name: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", size: "194" }
    at subfile-exchange/src/publisher.rs:51

  2023-11-13T20:54:01.523533Z  INFO crate::file_hasher: write_chunk_file, read_dir: "./example-file/", file_name: "example-create-17686085.dbin"
    at subfile-exchange/src/file_hasher.rs:115

  2023-11-13T20:54:01.525699Z DEBUG crate::file_hasher: Chunked file, file: "./example-file//example-create-17686085.dbin", total_bytes: 1052737, num_chunks: 2
    at subfile-exchange/src/file_hasher.rs:51

  2023-11-13T20:54:01.609974Z DEBUG crate::file_hasher: Chunk hash, hash: [184, 160, 246, 197, 215, 233, 213, 107, 51, 186, 242, 15, 214, 113, 77, 53, 54, 40, 103, 188, 239, 155, 98, 151, 232, 161, 68, 151, 12, 1, 125, 210]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:54:01.610021Z DEBUG crate::file_hasher: Chunk hash, hash_str: "uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:54:01.610260Z DEBUG crate::file_hasher: Chunk hash, hash: [78, 187, 172, 71, 70, 126, 17, 136, 55, 222, 142, 10, 69, 113, 175, 72, 223, 117, 211, 38, 175, 10, 72, 195, 236, 173, 233, 96, 137, 134, 101, 164]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:54:01.610274Z DEBUG crate::file_hasher: Chunk hash, hash_str: "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:54:01.610292Z TRACE crate::file_hasher: Created chunk file, file: ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }
    at subfile-exchange/src/file_hasher.rs:123

  2023-11-13T20:54:02.132445Z DEBUG crate::publisher: Added yaml file to IPFS, added: AddResponse { name: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv", size: "200" }
    at subfile-exchange/src/publisher.rs:51

  2023-11-13T20:54:02.133021Z  INFO crate::publisher: hash_and_publish_files, meta_info: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }]
    at subfile-exchange/src/publisher.rs:121

  2023-11-13T20:54:02.356114Z  INFO crate::publisher: Published subfile manifest to IPFS with hash: QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw
    at subfile-exchange/src/publisher.rs:128

  2023-11-13T20:54:02.356867Z  INFO subfile_cli: Published, result: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw"
    at subfile-exchange/src/main.rs:49



```



### Server

Serve multiple files in multiple subfile

## To add
| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `SUBFILE_SERVICE_NETWORK_SUBGRAPH_DEPLOYMENT` | `--network-subgraph-deployment` | `QmVQrrgeGGHEqRdjAByeLvnNnDMjdt85jZZB5EFZk62JWs` (`mainnet`) |
| `SUBFILE_SERVICE_NETWORK_SUBGRAPH_ENDPOINT`   | `--network-subgraph-endpoint`   | `https://api.thegraph.com/subgraphs/name/graphprotocol/graph-network-mainnet`           |
| `SUBFILE_SERVICE_METRICS_PORT`       | `--metrics-port`       |  Default: `7200`    |
| `SUBFILE_SERVICE_CLIENT_SIGNER_ADDRESS`       | `--client-signer-address`       |  `0x982D10c56b8BBbD6e09048F5c5f01b43C65D5aE0`    |
| `SUBFILE_SERVICE_LOG_LEVEL`       | `--log-level`       |  Default: `info`    |
| `SUBFILE_SERVICE_ALLOCATION_SYNCING_INTERVAL`       | `--allocation-syncing-interval`       |  Default: `120000`    |
| `SUBFILE_SERVICE_ESCROW_SUBGRAPH_ENDPOINT`       | `--escrow-subgraph-endpoint`       |  `'https://api.studio.thegraph.com/proxy/53925/eth-goerli-tap-subgraph/version/latest/'`    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--escrow-syncing-interval`       |  Default: `120000`    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--receipts-verifier-chain-id`       |  1, 5    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--receipts-verifier-address `       |  '0xD46c60558F7960407F4D00098145D77Fd061aD90'    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--rav-request-trigger-value`       |  Default: `10000000000000000000`    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--rav-request-timestamp-buffer-ns`       |  Default: `1000000000`    |
| `SUBFILE_SERVICE_ESCROW_SYNCING_INTERVAL`       | `--sender-aggregator-endpoints-file`       |  Default: `"aggregators.yaml"`    |

Consider access such as to postgres or files requiring authentication

```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange server \
  --host 0.0.0.0 \
  --port 5678 \
  --mnemonic "blah" \
  --subfiles "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw:./example-file/"

  2023-11-13T20:56:37.038316Z  INFO subfile_cli: Running cli, cli: Cli { role: Server(ServerArgs { host: "0.0.0.0", port: 5678, subfiles: ["QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw:./example-file/"] }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-exchange/src/main.rs:16

  2023-11-13T20:56:37.042506Z  INFO subfile_cli: Tracker request, server: ServerArgs { host: "0.0.0.0", port: 5678, subfiles: ["QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw:./example-file/"] }
    at subfile-exchange/src/main.rs:57

  2023-11-13T20:56:37.042762Z DEBUG crate::subfile_server: Validated subfile entries, entries: [("QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", "./example-file/")]
    at subfile-exchange/src/subfile_server/mod.rs:79

  2023-11-13T20:56:37.044076Z  INFO crate::subfile_server::util: Running package version PackageVersion {
    version: "0.0.1",
    dependencies: {},
}
    at subfile-exchange/src/subfile_server/util.rs:56

  2023-11-13T20:56:37.594728Z  INFO crate::subfile_reader: Read file content, content: Mapping {"files": Sequence [Mapping {"name": String("example0017686312.dbin"), "hash": String("QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ")}, Mapping {"name": String("example-create-17686085.dbin"), "hash": String("QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv")}]}
    at subfile-exchange/src/subfile_reader.rs:24

  2023-11-13T20:56:37.595171Z DEBUG crate::subfile_reader: subfile manifest, subfile: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }
    at subfile-exchange/src/subfile_reader.rs:31

  2023-11-13T20:56:37.595636Z DEBUG crate::subfile_reader: Fetch chunk file from IPFS, ipfs_hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ"
    at subfile-exchange/src/subfile_reader.rs:50

  2023-11-13T20:56:37.799012Z  INFO crate::subfile_reader: Read file content, content: Mapping {"file_name": String("example0017686312.dbin"), "total_bytes": Number(1508787), "chunk_size": Number(1048576), "chunk_hashes": Sequence [String("yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="), String("vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c=")]}
    at subfile-exchange/src/subfile_reader.rs:59

  2023-11-13T20:56:37.800539Z DEBUG crate::subfile_reader: Fetch chunk file from IPFS, ipfs_hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv"
    at subfile-exchange/src/subfile_reader.rs:50

  2023-11-13T20:56:37.996021Z  INFO crate::subfile_reader: Read file content, content: Mapping {"file_name": String("example-create-17686085.dbin"), "total_bytes": Number(1052737), "chunk_size": Number(1048576), "chunk_hashes": Sequence [String("uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI="), String("TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ=")]}
    at subfile-exchange/src/subfile_reader.rs:59

  2023-11-13T20:56:37.996495Z DEBUG crate::subfile_server: Read subfile, subfile: Subfile { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", local_path: "./example-file/", manifest: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }, chunk_files: [ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }, ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }] }
    at subfile-exchange/src/subfile_server/mod.rs:94

  2023-11-13T20:56:37.997204Z  INFO crate::subfile_server: Server listening on https://0.0.0.0:5678
    at subfile-exchange/src/subfile_server/mod.rs:66

  2023-11-13T20:57:45.443240Z TRACE crate::subfile_server: Received request
    at subfile-exchange/src/subfile_server/mod.rs:109

  2023-11-13T20:57:45.443240Z TRACE crate::subfile_server: Received request
    at subfile-exchange/src/subfile_server/mod.rs:109

  2023-11-13T20:57:45.444357Z DEBUG crate::subfile_server: Received file range request
    at subfile-exchange/src/subfile_server/mod.rs:126

  2023-11-13T20:57:45.444495Z DEBUG crate::subfile_server: Received file range request
    at subfile-exchange/src/subfile_server/mod.rs:126

  2023-11-13T20:57:45.444839Z DEBUG crate::subfile_server: Received file range request, subfiles: ServerState { subfiles: {"QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw": Subfile { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", local_path: "./example-file/", manifest: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }, chunk_files: [ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }, ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }] }}, release: PackageVersion { version: "0.0.1", dependencies: {} } }, id: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw"
    at subfile-exchange/src/subfile_server/mod.rs:130

  2023-11-13T20:57:45.445315Z DEBUG crate::subfile_server: Parse content range header
    at subfile-exchange/src/subfile_server/mod.rs:169

  2023-11-13T20:57:45.445788Z DEBUG crate::subfile_server::range: Serve file range, file_path: "./example-file/example0017686312.dbin", start_byte: 1048576, end_byte: 1508786
    at subfile-exchange/src/subfile_server/range.rs:44

  2023-11-13T20:57:45.446019Z DEBUG crate::subfile_server::range: Range validity check, start: 1048576, end: 1508786, file_size: 1508787
    at subfile-exchange/src/subfile_server/range.rs:71

  2023-11-13T20:57:45.446409Z TRACE crate::subfile_server::range: File seek to start at 1048576
    at subfile-exchange/src/subfile_server/range.rs:89

  2023-11-13T20:57:45.449501Z DEBUG crate::subfile_server: Received file range request, subfiles: ServerState { subfiles: {"QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw": Subfile { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", local_path: "./example-file/", manifest: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }, chunk_files: [ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }, ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }] }}, release: PackageVersion { version: "0.0.1", dependencies: {} } }, id: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw"
    at subfile-exchange/src/subfile_server/mod.rs:130

  2023-11-13T20:57:45.449681Z DEBUG crate::subfile_server: Parse content range header
    at subfile-exchange/src/subfile_server/mod.rs:169

  2023-11-13T20:57:45.449708Z DEBUG crate::subfile_server::range: Serve file range, file_path: "./example-file/example0017686312.dbin", start_byte: 0, end_byte: 1048575
    at subfile-exchange/src/subfile_server/range.rs:44

  2023-11-13T20:57:45.449751Z DEBUG crate::subfile_server::range: Range validity check, start: 0, end: 1048575, file_size: 1508787
    at subfile-exchange/src/subfile_server/range.rs:71

  2023-11-13T20:57:45.449773Z TRACE crate::subfile_server::range: File seek to start at 0
    at subfile-exchange/src/subfile_server/range.rs:89

  2023-11-13T20:57:45.728403Z TRACE crate::subfile_server: Received request
    at subfile-exchange/src/subfile_server/mod.rs:109

  2023-11-13T20:57:45.728403Z TRACE crate::subfile_server: Received request
    at subfile-exchange/src/subfile_server/mod.rs:109

  2023-11-13T20:57:45.728534Z DEBUG crate::subfile_server: Received file range request
    at subfile-exchange/src/subfile_server/mod.rs:126

  2023-11-13T20:57:45.728565Z DEBUG crate::subfile_server: Received file range request
    at subfile-exchange/src/subfile_server/mod.rs:126

  2023-11-13T20:57:45.728603Z DEBUG crate::subfile_server: Received file range request, subfiles: ServerState { subfiles: {"QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw": Subfile { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", local_path: "./example-file/", manifest: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }, chunk_files: [ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }, ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }] }}, release: PackageVersion { version: "0.0.1", dependencies: {} } }, id: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw"
    at subfile-exchange/src/subfile_server/mod.rs:130

  2023-11-13T20:57:45.731898Z DEBUG crate::subfile_server: Parse content range header
    at subfile-exchange/src/subfile_server/mod.rs:169

  2023-11-13T20:57:45.731985Z DEBUG crate::subfile_server::range: Serve file range, file_path: "./example-file/example-create-17686085.dbin", start_byte: 1048576, end_byte: 1052736
    at subfile-exchange/src/subfile_server/range.rs:44

  2023-11-13T20:57:45.732090Z DEBUG crate::subfile_server::range: Range validity check, start: 1048576, end: 1052736, file_size: 1052737
    at subfile-exchange/src/subfile_server/range.rs:71

  2023-11-13T20:57:45.732132Z TRACE crate::subfile_server::range: File seek to start at 1048576
    at subfile-exchange/src/subfile_server/range.rs:89

  2023-11-13T20:57:45.732498Z DEBUG crate::subfile_server: Received file range request, subfiles: ServerState { subfiles: {"QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw": Subfile { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", local_path: "./example-file/", manifest: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }, chunk_files: [ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }, ChunkFile { file_name: "example-create-17686085.dbin", total_bytes: 1052737, chunk_size: 1048576, chunk_hashes: ["uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI=", "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="] }] }}, release: PackageVersion { version: "0.0.1", dependencies: {} } }, id: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw"
    at subfile-exchange/src/subfile_server/mod.rs:130

  2023-11-13T20:57:45.732611Z DEBUG crate::subfile_server: Parse content range header
    at subfile-exchange/src/subfile_server/mod.rs:169

  2023-11-13T20:57:45.732639Z DEBUG crate::subfile_server::range: Serve file range, file_path: "./example-file/example-create-17686085.dbin", start_byte: 0, end_byte: 1048575
    at subfile-exchange/src/subfile_server/range.rs:44

  2023-11-13T20:57:45.732688Z DEBUG crate::subfile_server::range: Range validity check, start: 0, end: 1048575, file_size: 1052737
    at subfile-exchange/src/subfile_server/range.rs:71

  2023-11-13T20:57:45.732719Z TRACE crate::subfile_server::range: File seek to start at 0
    at subfile-exchange/src/subfile_server/range.rs:89


``````


Corresponding client

```
➜  ~ curl -H "Range: bytes=0-102000003" http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX
> range out of bound

➜  ~ curl -H "Range: bytes=0-1023" http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX --output -  
> [partial file...]

➜  ~ curl http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX 
> [whole file...]
```



### Downloader

```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange downloader \
   --ipfs-hash QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw \
   --gateway-url http://localhost:5678/subfiles/id/

  2023-11-13T20:57:44.211311Z  INFO subfile_cli: Downloader request, config: DownloaderArgs { ipfs_hash: "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", gateway_url: "http://localhost:5678/subfiles/id/", indexer_endpoints: [], output_dir: "./example-download" }
    at subfile-exchange/src/main.rs:26

  2023-11-13T20:57:44.885806Z  INFO crate::subfile_reader: Read file content, content: Mapping {"files": Sequence [Mapping {"name": String("example0017686312.dbin"), "hash": String("QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ")}, Mapping {"name": String("example-create-17686085.dbin"), "hash": String("QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv")}]}
    at subfile-exchange/src/subfile_reader.rs:24

  2023-11-13T20:57:44.886232Z DEBUG crate::subfile_reader: subfile manifest, subfile: SubfileManifest { files: [FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }, FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }] }
    at subfile-exchange/src/subfile_reader.rs:31

  2023-11-13T20:57:44.886955Z DEBUG crate::subfile_client: Download chunk file, info: FileMetaInfo { name: "example0017686312.dbin", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ" }
    at subfile-exchange/src/subfile_client/mod.rs:109

  2023-11-13T20:57:44.887215Z DEBUG crate::subfile_reader: Fetch chunk file from IPFS, ipfs_hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ"
    at subfile-exchange/src/subfile_reader.rs:50

  2023-11-13T20:57:45.425129Z  INFO crate::subfile_reader: Read file content, content: Mapping {"file_name": String("example0017686312.dbin"), "total_bytes": Number(1508787), "chunk_size": Number(1048576), "chunk_hashes": Sequence [String("yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="), String("vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c=")]}
    at subfile-exchange/src/subfile_reader.rs:59

  2023-11-13T20:57:45.426529Z TRACE crate::subfile_client: Download chunk index, i: 0
    at subfile-exchange/src/subfile_client/mod.rs:140

  2023-11-13T20:57:45.426912Z TRACE crate::subfile_client: Download chunk index, i: 1
    at subfile-exchange/src/subfile_client/mod.rs:140

  2023-11-13T20:57:45.428446Z DEBUG crate::subfile_client: Make range request, query_endpoint: "http://localhost:5678/subfiles/id/QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", range: "bytes=0-1048575"
    at subfile-exchange/src/subfile_client/mod.rs:220

  2023-11-13T20:57:45.428707Z DEBUG crate::subfile_client: Make range request, query_endpoint: "http://localhost:5678/subfiles/id/QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", range: "bytes=1048576-1508786"
    at subfile-exchange/src/subfile_client/mod.rs:220

  2023-11-13T20:57:45.486215Z DEBUG crate::file_hasher: Chunk hash, hash: [191, 27, 225, 27, 22, 159, 177, 171, 252, 216, 173, 16, 107, 156, 16, 160, 148, 174, 246, 237, 228, 234, 254, 114, 77, 246, 129, 45, 195, 205, 203, 231]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:57:45.486334Z DEBUG crate::file_hasher: Chunk hash, hash_str: "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:57:45.519673Z DEBUG crate::file_hasher: Chunk hash, hash: [202, 227, 94, 140, 125, 254, 145, 68, 217, 119, 24, 19, 178, 40, 120, 144, 232, 4, 19, 190, 183, 234, 100, 36, 76, 83, 65, 117, 96, 23, 26, 146]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:57:45.519710Z DEBUG crate::file_hasher: Chunk hash, hash_str: "yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:57:45.519972Z  INFO crate::subfile_client: chunk_download_result: ()
    at subfile-exchange/src/subfile_client/mod.rs:169

  2023-11-13T20:57:45.520002Z  INFO crate::subfile_client: chunk_download_result: ()
    at subfile-exchange/src/subfile_client/mod.rs:169

  2023-11-13T20:57:45.520485Z DEBUG crate::subfile_client: Download chunk file, info: FileMetaInfo { name: "example-create-17686085.dbin", hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv" }
    at subfile-exchange/src/subfile_client/mod.rs:109

  2023-11-13T20:57:45.520504Z DEBUG crate::subfile_reader: Fetch chunk file from IPFS, ipfs_hash: "QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv"
    at subfile-exchange/src/subfile_reader.rs:50

  2023-11-13T20:57:45.726127Z  INFO crate::subfile_reader: Read file content, content: Mapping {"file_name": String("example-create-17686085.dbin"), "total_bytes": Number(1052737), "chunk_size": Number(1048576), "chunk_hashes": Sequence [String("uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI="), String("TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ=")]}
    at subfile-exchange/src/subfile_reader.rs:59

  2023-11-13T20:57:45.726644Z TRACE crate::subfile_client: Download chunk index, i: 0
    at subfile-exchange/src/subfile_client/mod.rs:140

  2023-11-13T20:57:45.726751Z TRACE crate::subfile_client: Download chunk index, i: 1
    at subfile-exchange/src/subfile_client/mod.rs:140

  2023-11-13T20:57:45.726820Z DEBUG crate::subfile_client: Make range request, query_endpoint: "http://localhost:5678/subfiles/id/QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", range: "bytes=0-1048575"
    at subfile-exchange/src/subfile_client/mod.rs:220

  2023-11-13T20:57:45.726821Z DEBUG crate::subfile_client: Make range request, query_endpoint: "http://localhost:5678/subfiles/id/QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw", range: "bytes=1048576-1052736"
    at subfile-exchange/src/subfile_client/mod.rs:220

  2023-11-13T20:57:45.734061Z DEBUG crate::file_hasher: Chunk hash, hash: [78, 187, 172, 71, 70, 126, 17, 136, 55, 222, 142, 10, 69, 113, 175, 72, 223, 117, 211, 38, 175, 10, 72, 195, 236, 173, 233, 96, 137, 134, 101, 164]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:57:45.734187Z DEBUG crate::file_hasher: Chunk hash, hash_str: "TrusR0Z+EYg33o4KRXGvSN910yavCkjD7K3pYImGZaQ="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:57:45.812445Z DEBUG crate::file_hasher: Chunk hash, hash: [184, 160, 246, 197, 215, 233, 213, 107, 51, 186, 242, 15, 214, 113, 77, 53, 54, 40, 103, 188, 239, 155, 98, 151, 232, 161, 68, 151, 12, 1, 125, 210]
    at subfile-exchange/src/file_hasher.rs:26

  2023-11-13T20:57:45.812480Z DEBUG crate::file_hasher: Chunk hash, hash_str: "uKD2xdfp1WszuvIP1nFNNTYoZ7zvm2KX6KFElwwBfdI="
    at subfile-exchange/src/file_hasher.rs:29

  2023-11-13T20:57:45.812641Z  INFO crate::subfile_client: chunk_download_result: ()
    at subfile-exchange/src/subfile_client/mod.rs:169

  2023-11-13T20:57:45.812663Z  INFO crate::subfile_client: chunk_download_result: ()
    at subfile-exchange/src/subfile_client/mod.rs:169

  2023-11-13T20:57:45.813860Z  INFO crate::subfile_client: Chunk files download results, completed_files: [Ok(()), Ok(())]
    at subfile-exchange/src/subfile_client/mod.rs:96

  2023-11-13T20:57:45.814023Z  INFO subfile_cli: Download result: Ok(
    (),
)
    at subfile-exchange/src/main.rs:40

➜  subfile-exchange git:(main) ✗ 
```

