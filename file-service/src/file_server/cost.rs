use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use super::{FileServiceError, ServerContext};

/// Endpoint for cost to download per byte
pub async fn cost(
    State(context): State<ServerContext>,
) -> Result<impl IntoResponse, FileServiceError> {
    let price = context.state.lock().await.price_per_byte.to_string();

    Ok(Json(json!({"data": price})))
}

// use std::str::FromStr;
// use std::sync::Arc;

// use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
// use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
// use axum::extract::State;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use thegraph::types::DeploymentId;

// use crate::database::{self, CostModel};

// #[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
// pub struct GraphQlCostModel {
//     pub deployment: String,
//     pub model: Option<String>,
//     pub variables: Option<Value>,
// }

// impl From<CostModel> for GraphQlCostModel {
//     fn from(model: CostModel) -> Self {
//         Self {
//             deployment: model.deployment.to_string(),
//             model: model.model,
//             variables: model.variables,
//         }
//     }
// }

// #[derive(Default)]
// pub struct Query;

// #[Object]
// impl Query {
//     async fn cost_models(
//         &self,
//         ctx: &Context<'_>,
//         deployments: Vec<String>,
//     ) -> Result<Vec<GraphQlCostModel>, anyhow::Error> {
//         let deployment_ids = deployments
//             .into_iter()
//             .map(|s| DeploymentId::from_str(&s))
//             .collect::<Result<Vec<DeploymentId>, _>>()?;
//         let pool = &ctx.data_unchecked::<Arc<ServiceState>>().database;
//         let cost_models = database::cost_models(pool, &deployment_ids).await?;
//         Ok(cost_models.into_iter().map(|m| m.into()).collect())
//     }

//     async fn cost_model(
//         &self,
//         ctx: &Context<'_>,
//         deployment: String,
//     ) -> Result<Option<GraphQlCostModel>, anyhow::Error> {
//         let deployment_id = DeploymentId::from_str(&deployment)?;
//         let pool = &ctx.data_unchecked::<Arc<ServiceState>>().database;
//         database::cost_model(pool, &deployment_id)
//             .await
//             .map(|model_opt| model_opt.map(GraphQlCostModel::from))
//     }
// }

// pub type CostSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// pub async fn build_schema() -> CostSchema {
//     Schema::build(Query, EmptyMutation, EmptySubscription).finish()
// }

// pub async fn cost(
//     State(state): State<Arc<ServiceState>>,
//     req: GraphQLRequest,
// ) -> GraphQLResponse {
//     state
//         .cost_schema
//         .execute(req.into_inner().data(state.clone()))
//         .await
//         .into()
// }
