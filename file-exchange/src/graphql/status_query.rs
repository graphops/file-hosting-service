use anyhow::Result;
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

use super::{graphql_query, Query};

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQlBundle {
    pub ipfs_hash: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FileStatus {
    bundles: Vec<GraphQlBundle>,
}

// Indexer bundles
//TODO: find how to generate bundle GraphQL type shareable to file-service
//so values can be parsed more elegantly
pub async fn indexer_bundles(client: &reqwest::Client, url: &str) -> Result<Vec<String>, Error> {
    let status_url = format!("{}/files-status", url);
    let query = r#"query{bundles{ipfsHash}}"#;
    let result = graphql_query::<FileStatus>(client, &status_url, Query::new(query)).await?;

    Ok(result
        .map_err(Error::GraphQLResponseError)?
        .bundles
        .iter()
        .map(|bundle| bundle.ipfs_hash.clone())
        .collect::<Vec<String>>())
}
