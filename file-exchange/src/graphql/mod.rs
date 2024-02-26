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

use crate::errors::Error;

pub mod cost_query;
pub mod escrow_query;
pub mod network_query;
pub mod status_query;

//TODO: deprecate when we determine the on-chain discovery piece for allocation
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

pub async fn graphql_query<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    query_url: &str,
    query: impl IntoRequestParameters + Send,
) -> Result<ResponseResult<T>, Error> {
    client
        .post(query_url)
        .header(header::USER_AGENT, "file-service")
        .send_graphql(query)
        .await
        .map_err(Error::GraphQLRequestError)
}
