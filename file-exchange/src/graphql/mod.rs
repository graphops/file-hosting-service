use alloy_primitives::Address;
use graphql_http::{
    graphql::{Document, IntoDocument},
    http::request::{IntoRequestParameters, RequestParameters},
    http_client::{ReqwestExt, ResponseResult},
};
use reqwest::header;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::str::FromStr;

pub mod cost_query;
pub mod network_query;
// This file should contain graphql queries for
// network_subgraph: fetching registered_indexers' public_url, indexer file hosting allocation
// escrow_subgraph: escrow account(sender,receiver) balance

pub fn allocation_id(_indexer: &str) -> Address {
    Address::from_str("0x29cc405f6104b1d6d2d7f2989c5932818f6268c2").unwrap()
}

#[derive(Clone)]
pub struct Query {
    pub query: Document,
    pub variables: Map<String, Value>,
}

impl Query {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.into_document(),
            variables: Map::default(),
        }
    }

    pub fn new_with_variables(
        query: impl IntoDocument,
        variables: impl Into<QueryVariables>,
    ) -> Self {
        Self {
            query: query.into_document(),
            variables: variables.into().into(),
        }
    }
}

pub struct QueryVariables(Map<String, Value>);

impl<'a, T> From<T> for QueryVariables
where
    T: IntoIterator<Item = (&'a str, Value)>,
{
    fn from(variables: T) -> Self {
        Self(
            variables
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<Map<_, _>>(),
        )
    }
}

impl From<QueryVariables> for Map<String, Value> {
    fn from(variables: QueryVariables) -> Self {
        variables.0
    }
}

impl IntoRequestParameters for Query {
    fn into_request_parameters(self) -> RequestParameters {
        RequestParameters {
            query: self.query.into_document(),
            variables: self.variables,
            extensions: Map::default(),
            operation_name: None,
        }
    }
}

pub struct GraphQLClient {
    http_client: reqwest::Client,
}

impl Default for GraphQLClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphQLClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query_url: &str,
        query: impl IntoRequestParameters + Send,
    ) -> Result<ResponseResult<T>, anyhow::Error> {
        Ok(self
            .http_client
            .post(query_url)
            .header(header::USER_AGENT, "file-service")
            .send_graphql(query)
            .await?)
    }
}
