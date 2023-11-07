## Subfile service specification 

Subfile service initialization could include a list of IPFS hashes for subfiels the service supports upon initialization

### CLI command for the seeding request should include 

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
templates:
  - kind: ethereum/contract
    mapping:
      abis:
        - file:
            /: /ipfs/Qmdf7nHjUMcRMvGEdqwV7WxzBu3FZ9k47w7Bd5dJjbz38q
          name: EpochManager
      apiVersion: 0.0.7
      entities:
        - TokenLockWallet
      eventHandlers:
        - event: 'TokensReleased(indexed address,uint256)'
          handler: handleTokensReleased
      file:
        /: /ipfs/QmUrL9HkFD2YbpZ6PzyQCtfVhCik23wsv27GKsoNvTbRHL
      kind: ethereum/events
      language: wasm/assemblyscript
    name: GraphTokenLockWallet
    network: arbitrum-goerli
    source:
      abi: GraphTokenLockWallet
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


### Calculations

Let total file size be F, chunk size be c, hash size be 256bits = 32bytes

F=5TB, c=1MB => ~5million chunks, 160 MB for hashes not including positioning
F=5TB, c=10MB, lower bound by file size 25MB => 16-19.2MB for hashes not including positioning

Merkel proof with roots



