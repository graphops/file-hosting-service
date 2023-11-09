## Subfile manfiest specifications

Document the structure of subfile and chunk files.




### Calculations

Let total file size be F, chunk size be c, hash size be 256bits = 32bytes

F=5TB, c=1MB => ~5million chunks, 160 MB for hashes not including positioning
F=5TB, c=10MB, lower bound by file size 25MB => 16-19.2MB for hashes not including positioning

Merkel proof with roots


#### Some real-life numbers 
Total firehose size for Ethereum = 1.1TiB
Files are 100 blocks each

[18471362](https://etherscan.io/block/18471362) blocks = ~18471 files of 100 blocks each

1.1TiB / 18471 = avg file size = 0.06 GiB 

chunk size = 64MB would be 1 chunk per file

so then split chunk size to something smaller like 2Mb, play around with this, leave it configurable


Subfile Manfiest 
```
dataSources: // list of files
  - kind: ethereum/flatfile // the kind of files shared
    providerVersion: [version] // version used by indexing method; i.e. firehose versioning
    provider: firehose
    source: // chunk files
      chunkFile: 
        startBlock: ...
        endBlock: ...
        pieceLength: // number of bytes per piece
        length: // Total bytes of the file 
        rootHash: // markle root hash for the file
        /: /ipfs/[Qm...] 
    language: [...]
    name: EthereumFirehose
    network: ethereum
description: "..."
features:
  - Tracing
publisher_url: [persisted url of the publisher status]
specVersion: [subfile version]
```




abis file

```
[
  {
    "inputs": [],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        ...
      }, ...
    ]
  },...
]
```

...


### CLI command for the populating manifest

One of these is required, conflict with each other
file_path: Option<String>,   // Path to the file/directory to seed
file_link: Option<String>,   // Previously generated magnet link

These can be interactive, subcommand, or parsed from a config file
name: Option<String>,        // Name to give the torrent file
file_type: String,           // flatfiles and such, TODO: replace with an enum for supported types
file_version: String,        // User specify the torrent version
identifier: String,          // Describe a commonly available unit (firehose Ethereum chain, or subgrpah deployment hash, ...)
start_block: Option<u64>,    // Flatfiles require a start block
end_block: Option<u64>,      // Flatfiles require an end block, snapshots can utilize it as target_block
trackers: Vec<String>, // A list of trackers to announce data availability to, we should provide a set of defaults
subfile_store_path: String, // The path to store subfile.yaml once it has been generated



### Current manifest


#### Subfile manifest

https://ipfs.network.thegraph.com/api/v0/cat?arg=QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX
```
files:
- path: ./example-file/example0017686312.dbin
  hash: Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC
```

#### Chunk file schema

https://ipfs.network.thegraph.com/api/v0/cat?arg=Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC
```
merkle_root: cf3726c4ffb3a36a0cbe955fd4ecea4ec17a075c86dd98983778abfa9a0bfcb4
chunks:
- cf3726c4ffb3a36a0cbe955fd4ecea4ec17a075c86dd98983778abfa9a0bfcb4
- ...
```
^ need fixing





### Referencing subgraph.yaml

https://ipfs.network.thegraph.com/api/v0/cat?arg=Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX

manifest
```
dataSources:
  - kind: ethereum/contract
    mapping:
      abis:
        - file:
            /: /ipfs/QmQoSyqRFYk12SDrEXDo3d1pwGzNwMkbu6xRjscZZrpeoi
          name: Controller
      apiVersion: 0.0.7
      entities:
        - Indexer
      eventHandlers:
        - event: 'SetContractProxy(indexed bytes32,address)'
          handler: handleSetContractProxy
      file:
        /: /ipfs/QmYQm7DcFCtq7erXuH4obV7e3nMAphVqrCiDVGEck3Hyc8
      kind: ethereum/events
      language: wasm/assemblyscript
    name: Controller
    network: arbitrum-goerli
    source:
      abi: Controller
      address: '0x7f734E995010Aa8d28b912703093d532C37b6EAb'
      startBlock: 1023264
  - kind: ...
description: The Graph Network Smart Contracts on Ethereum
features:
  - ipfsOnEthereumContracts
  - fullTextSearch
repository: 'https://github.com/graphprotocol/graph-network-subgraph'
schema:
  file:
    /: /ipfs/QmVWxUnF6vxf4xUfrg6ferLr2tU6iAsY7wJBmtzQpqu3rd
specVersion: 0.0.5
```