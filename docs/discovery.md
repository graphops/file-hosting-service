# File Discovery

With various packaging methods, types of files, and other types of combinations, we stay focused on discovering and matching file CIDs on/off-chain.

## Off-chain approach

Indexers serve `/status` endpoint that provides a list of Manifest IPFS hashes, representing the list of available lists the local indexer is serving. This is sufficient for matching specific manifests for bundles, later on we will allow matching for single file manifests.

On both **Bundle/File** level, the discovery is relatively straightforward for the client, given that they have choosen a Manifest IPFS hash to download (`target_manifest`).

1. Client provide a status list of `indexer_endpoints`. Later on, we can add an automatic mode of grabbing all registered indexer service url from the registery contract.

2. Client pings `/operator` and `/status` endpoint for all indexer endpoints. `/operator` will provide the indexer operator and `/status` endpoint will provide the indexer's available manifests.

    a. if `target_manifest` is in the available manifests, collect indexer operator and endpoint as an available service

3. Collect a list of available services. Returns early if the list is empty.

If discovery is matching on a `Bundle` level, we further consider matching files across bundle manifests so that consumers can be prompted with alternatives if the `target_manifest` is unavailable. This increases file availability by decreasing the criteria for matching a bundle.

Imagine a server serving $bundle_a = {file_x, file_y, file_z}$. Client requests $bundle_b = {file_x}$. The Bundle IPFS hash check will determine that $bundle_a\neq bundle_b$. We add an additional check to resolve $bundle_a$ and $bundle_b$ to file manifest hashes for matching. 

1. Query the content of `target_bundle` for its vector of file manifest hashes

2. Query the content of bundles served by indexers, create a nested map of indexer to bundles to files: `Map<Indexer, Map<Bundle, Files>>`.

3. For each `target_bundle` file manifest, check if there is an indexer serving a bundle that contains the target file. Record the indexer and bundle hash, indexed by the file hash.

4. if there is a target file unavailable from all indexers, immediately return unavailability as the `target_manifest` cannot be completed.

5. return the recorded map of file to queriable `indexer_endpoint` and manifest hash for the user evaluation

Later on, we may generate a summary of which manifest has the highest percentage of compatibility. The further automated approach will consist of client taking the recorded availability map and construct range download requests based on the corresponding indexer_endpoint, server manifest, and file hash.

In the diagram below, keep in mind that it is possible for IPFS files (schema files) to be hosted by indexer services as well, which will remove the necessity of using an IPFS gateway. However, for the sake of simplicity and accuracy to the current state of the project, we keep the IPFS gateway component required. 

```mermaid
graph LR
    I[Indexer] -->|post schema| IPFS[IPFS Gateway]
    I -->|post schema <br> manage availability| I 
    C[Client] -.->|schema| IPFS
    C -->|determine Bundle hash| C 
    C -.->|schema| I
    C -.->|availability| I
    C -->|paid query| I
```
With an explorer, we can imagine a future discovery user interface along the lines of 

```mermaid
graph LR
    I[Indexer] -->|manage availability| I
    I[Indexer] -->|post schema| IPFS[IPFS Gateway]
    E[Explorer] -.->|availability| I
    E -.->|query scehma| IPFS
    C[Client] -.->|select Bundle| E
    C -->|authorize| E
    E -->|paid query| I
    E -->|respond| C
```  

## On-chain approach (unrealized)


**On-chain portion**

Indexers registers their server url at the registry contract, as this step is crucial in making the Indexer's service discoverable and accessible within the network. We assume Indexer has already registered (through indexer-agent). 

Indexers are expected to create explicit allocation against a specific IPFS hash. The hashes uniquely identify bundles, acting as a unit identifiable and verifiable between the Indexers and the data requested by consumers. This process ensures that data retrieval is efficiently managed and that Indexers are appropriately allocated to serve specific data needs.

The network subgraph keeps track of all registered Indexers and their active (and closed) allocations. We assume an update to the network subgraph such that `SubgraphDeployment` field gets renamed or encasuplated to a more generic entity such as `DataServiceDeployment` with an additional `dataService` enum field. This addition is essential for querying and filtering information about deployments.

- Identify Available Bundles: Clients can view all bundles currently available in the network, along with the Indexers allocated to these bundles.
- Query Specific Bundles: Once a desired Bundle is identified, clients can make targeted queries pertaining to that Bundle and the Indexers actively allocated to it.

With the updated network subgraph, on-chain discovery can be done with flexible queries.

```graphql
query {
// Discover through Bundle IPFS hash
subgraphDeployments(where:{ipfsHash: $deployment_ipfs_hash}){
    id
    ipfsHash
    indexerAllocations {
        indexer{
        id
        url
        }
    }
}

// Discover through indexers
indexers {
    allocations(where: {
        dataServiceType: File,
        fileDeployment: $deployment_ipfs_hash
    }) {
        id
        url
        allocatedTokens
        createdAt
        FileDeployment {
            ipfsHash
        }
    }
  }
}

```


**Off-chain approach**

Clients can discover all the available bundles through the network subgraph, and the allocated indexers. They are responsible for identifying the desired bundles and making query specific to the Bundle and the actively allocated indexers. To gain insights to an IPFS hash, the client might query the IPFS file content to read Bundle descriptions and file manifest hashes. 

A client may want to resolve all the available Bundle manifest to discover the best fit for their interest, or a client may decide to download a specific file instead of all the files contained in a Bundle. Discovery can be made through specific indexer service endpoints or IPFS gateways. 

```mermaid
graph TD
    subgraph "On-Chain"
        NS[Network Subgraph] -->|index| NC
    end
    subgraph "Off-Chain"
        I[Indexer] -->|registers server_url| NC[Network Contract]
        I[Indexer] -->|allocates ipfs_hash| NC[Network Contract]
        I[Indexer] -->|post schema| IPFS[IPFS Gateway]
        E[Explorer] -.->|availability| NS
        E -.->|query scehma| IPFS
        C[Client] -.->|select Bundle| E
        C -->|paid query| E
        E -->|routed paid query| I
        C -->|direct paid Query| I
    end
```

As we keep the diagram simple, it is possible to have indexer serve/host schema files as part of indexer service and become independent of IPFS gateway


## Trade-off

First assume that on-chain allocation does not bring significant economic guarantee to FHS (no rational slashing).

If there are $m$ data producers, $n$ data consumers, and $p$ datasets to discover, then 

**Offchain** 

The only on-chain cost occurred are indexer registration and/or server url allocation: $O(1)$.

Data consumers looking for 1 file will run a program with runtime complexity of $O(mp)$. Data producers will serve $O(n)$ queries for each consumers looking for a file. The discovery process mostly falls on data consumers.

**Onchain**

Data producers spend on indexer registration and per file allocation: $O(p)$. We expect the network subgraph to index the data and make available for queries (graphcast can acheive similar "oracle" functionality; particularly with challengers).

Data consumers looking for 1 file will run a program with runtime complexity of $O(1)$. Data producers will not need to serve status queries for p2p discovery.
