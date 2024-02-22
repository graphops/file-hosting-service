use anyhow::Result;
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

use super::{graphql_query, Query};

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GraphQlCostModel {
    pub price_per_byte: f32,
}

// Types for deserializing the file statuses response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CostData {
    cost_model: Option<GraphQlCostModel>,
}

pub async fn indexer_bundle_cost(
    client: &reqwest::Client,
    url: &str,
    deployment: &str,
) -> Result<Option<f32>, Error> {
    let cost_endpoint = format!("{}/files-cost", url);
    let query =
        r#"query cost($deployment: String!){costModel(deployment: $deployment){pricePerByte}}"#;
    let result = graphql_query::<CostData>(
        client,
        &cost_endpoint,
        Query::new_with_variables(query, [("deployment", deployment.into())]),
    )
    .await?;
    Ok(result
        .map_err(Error::GraphQLResponseError)?
        .cost_model
        .map(|c| c.price_per_byte))
}
