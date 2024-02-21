use serde::Deserialize;

use super::{GraphQLClient, Query};

// Query current epoch from network subgraph
pub async fn current_epoch(
    graphql_client: GraphQLClient,
    network_subgraph: &str,
    graph_network_id: u64,
) -> Result<u64, anyhow::Error> {
    // Types for deserializing the network subgraph response
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GraphNetworkData {
        graph_network: Option<GraphNetwork>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GraphNetwork {
        current_epoch: u64,
    }

    // Query the current epoch
    let query = r#"query epoch($id: ID!) { graphNetwork(id: $id) { currentEpoch } }"#;
    let result = graphql_client
        .query::<GraphNetworkData>(
            network_subgraph,
            Query::new_with_variables(query, [("id", graph_network_id.into())]),
        )
        .await?;

    result?
        .graph_network
        .ok_or_else(|| anyhow::anyhow!("Network {} not found", graph_network_id))
        .map(|network| network.current_epoch)
}
