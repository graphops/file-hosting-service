## To run an indexer-subfile server

To start, you would need to provide several configurations

### Requirements

An indexer running the server must provide configurations that enable the server to access the served files. The local paths are matched for individual subfiles, and subfiles should contain a list of file metainfo that specify a file name that is commonly agreed upon. (TODO: Regex matching can allow more varity of file names for separate indexers)

> Based on how packaging and verification schemes are set up, the server may need to generate merkle proofs along side range requests

> With TAP, Server should parse for receipts in the range request header. Receipts should be verified and stored and RAV be saved for (receiver, sender, subfile_id). Server can have configurable RAV collection requirements.


### CLI example
```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange server \
  --host 0.0.0.0 \
  --port 5678 \
  --mnemonic "blah" \
  --subfiles "QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw:./example-file/"
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
