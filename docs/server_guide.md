## To run an indexer-subfile server

To start, you would need to provide several configurations

### Requirements

An indexer running the server must provide configurations that enable the server to access the served files. The local paths are matched for individual subfiles, and subfiles should contain a list of file metainfo that specify a file name that is commonly agreed upon. (TODO: Regex matching can allow more varity of file names for separate indexers)

> Based on how packaging and verification schemes are set up, the server may need to generate merkle proofs along side range requests

> With TAP, Server should parse for receipts in the range request header. Receipts should be verified and stored and RAV be saved for (receiver, sender, subfile_id). Server can have configurable RAV collection requirements.


### CLI example
```
✗ cargo run -p subfile-exchange server \
  --host 0.0.0.0 \
  --port 5678 \
  --mnemonic "abondon abondon abondon abondon abondon abondon abondon abondon abondon abondon abondon abondon" \
  --admin-auth-token "imadmin" \
  --subfiles "QmY9aHuMqSPoLixVRdcYQei2cAtChBQNbjdtL5VzaQdFzw:./example-file/"
```

### Configuration matrix

| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `SUBFILE_SERVICE_MNEMONIC`                    | `--mnemonic`                    | Ethereum mnemonic for indexer operator           |
| `SUBFILE_SERVICE_ETHEREUM_NETWORK`            | `--ethereum-network`            | `mainnet`, `goerli`, `arbitrum-one`, `arbigrum-goerli`                              |
| `SUBFILE_SERVICE_INDEXER_ADDRESS`             | `--indexer-address`             | Ethereum address of the indexer              |
| `SUBFILE_SERVICE_FILES`             | `--files`             | IPFS hash of the initial subfiles and their location `QmHash1:filePath1,QmHash2:filePath2` TODO: Use an advanced configuration file for input               |

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

## Admin API

Support basic methods to get, add, and remove subfile services.

```
✗ curl http://localhost:5678/admin -X POST \    
  -H "Content-Type: application/json" \
  -H "AUTHORIZATION: Bearer imadmin" \
 --data '{"method":"add_subfile","params":["QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ:local_path"],"id":1,"jsonrpc":"2.0"}'
{"error":"Invalid local path: local_path"}%                                                  

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
