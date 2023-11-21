## Subfile manfiest specifications

Document the structure of subfile and chunk files.


### Packaging options

- Each flatfile is verified against a chunk_file which contains an ordered list of hashes for every chunk of data. Then, subfile manifest contains a map of flatfile name to chunk_file IPFS
- Each flatfile is verified against the root of a merkle tree, in which a chunk request can be verified through generating a merkle proof (if the merkele tree is posted publicly then anyone can be a proofer; otherwise the server must provide the proof). The subfile manifest contains a map for flatfile name to flatfile merkle root
- Each flatfile is verified against the root of a merkle tree, in which a chunk request can be verified through generating a merkle proof. The subfile manifest contains a merkle root of flatfile roots, in which the merkle tree is posted in a separate file. (Full Merkle tree should available for public access especially if individual flatfile tree is already public. Otherwise the server must serve all chunk and file checks).
- Each flatfile is verified against a chunk_file which contains an ordered list of hashes for every chunk of data. The subfile manifest contains a merkle root of chunk_file CID, in which the merkle tree is posted in a separate file.

In short

| | Ordered list | Merkle Tree | 
| --- | --- | --- | 
| File verification | $O(m^2)$ | $O(m\log(m))$ |
| File memory | $O(m)$ | $O(2m-1)$ |
| Package verification | $O(n^2)$ | $O(n\log(n))$ |
| Package memory | $O(n)$ | $O(2n-1)$ |

Where $m$ is the number of chunks for a file ($\frac{\text{file size}}{\text{chunk size}}$), $n$ is the number of files in a package.





### Subfile Manfiest 
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

https://ipfs.network.thegraph.com/api/v0/cat?arg=QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw
```
files:
- name: example0017686312.dbin
  hash: QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ
- name: example-create-17686085.dbin
  hash: QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv
- ...
```

#### Chunk file schema

https://ipfs.network.thegraph.com/api/v0/cat?arg=QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ
```
file_name: example0017686312.dbin
total_bytes: 1508787
chunk_size: 1048576
chunk_hashes:
- yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=
- vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c=
- ...
```



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