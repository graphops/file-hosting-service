use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use file_exchange::manifest::Bundle;
use serde::{Deserialize, Serialize};

use crate::file_server::ServerContext;

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct GraphQlCostModel {
    pub deployment: String,
    pub price_per_byte: f64,
}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    /// Provide an array of cost model to the queried deployment whether it is served or not
    async fn cost_models(
        &self,
        ctx: &Context<'_>,
        deployments: Vec<String>,
    ) -> Result<Vec<GraphQlCostModel>, anyhow::Error> {
        let price: f64 = ctx
            .data_unchecked::<ServerContext>()
            .state
            .config
            .server
            .price_per_byte;
        let cost_models = deployments
            .into_iter()
            .map(|s| GraphQlCostModel {
                deployment: s,
                price_per_byte: price,
            })
            .collect::<Vec<GraphQlCostModel>>();
        Ok(cost_models)
    }

    /// provide a cost model for a specific bundle served
    async fn cost_model(
        &self,
        ctx: &Context<'_>,
        deployment: String,
    ) -> Result<Option<GraphQlCostModel>, anyhow::Error> {
        let bundle: Option<Bundle> = ctx
            .data_unchecked::<ServerContext>()
            .state
            .bundles
            .lock()
            .await
            .get(&deployment)
            .cloned();
        let res = bundle.map(|_b| {
            let price: f64 = ctx
                .data_unchecked::<ServerContext>()
                .state
                .config
                .server
                .price_per_byte;
            GraphQlCostModel {
                deployment,
                price_per_byte: price,
            }
        });
        Ok(res)
    }
}

pub type CostSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub async fn build_schema() -> CostSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

pub async fn cost(State(context): State<ServerContext>, req: GraphQLRequest) -> GraphQLResponse {
    context
        .state
        .cost_schema
        .execute(req.into_inner().data(context.clone()))
        .await
        .into()
}
