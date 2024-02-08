// // Copyright 2023-, GraphOps and Semiotic Labs.
// // SPDX-License-Identifier: Apache-2.0

// use std::time::Duration;
// use std::{collections::HashSet, str::FromStr};

// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use sqlx::{postgres::PgPoolOptions, PgPool};
// use thegraph::types::{DeploymentId, DeploymentIdError};
// use tracing::debug;

// pub async fn connect(url: &str) -> PgPool {
//     debug!("Connecting to database");

//     PgPoolOptions::new()
//         .max_connections(50)
//         .acquire_timeout(Duration::from_secs(3))
//         .connect(url)
//         .await
//         .expect("Should be able to connect to the database")
// }

// /// Internal cost model representation as stored in the database.
// ///
// /// These can have "global" as the deployment ID.
// #[derive(Debug, Clone)]
// struct DbCostModel {
//     pub deployment: String,
//     pub model: Option<String>,
//     pub variables: Option<Value>,
// }

// /// External representation of cost models.
// ///
// /// Here, any notion of "global" is gone and deployment IDs are valid deployment IDs.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CostModel {
//     pub deployment: DeploymentId,
//     pub model: Option<String>,
//     pub variables: Option<Value>,
// }

// impl TryFrom<DbCostModel> for CostModel {
//     type Error = DeploymentIdError;

//     fn try_from(db_model: DbCostModel) -> Result<Self, Self::Error> {
//         Ok(Self {
//             deployment: DeploymentId::from_str(&db_model.deployment)?,
//             model: db_model.model,
//             variables: db_model.variables,
//         })
//     }
// }

// impl From<CostModel> for DbCostModel {
//     fn from(model: CostModel) -> Self {
//         let deployment = model.deployment;
//         DbCostModel {
//             deployment: format!("{deployment:#x}"),
//             model: model.model,
//             variables: model.variables,
//         }
//     }
// }

// /// Query cost models from the database, merging the global cost model in
// /// whenever there is no cost model defined for a deployment.
// pub async fn cost_models(
//     pool: &PgPool,
//     deployments: &[DeploymentId],
// ) -> Result<Vec<CostModel>, anyhow::Error> {
//     let hex_ids = deployments
//         .iter()
//         .map(|d| format!("{d:#x}"))
//         .collect::<Vec<_>>();

//     let mut models = if deployments.is_empty() {
//         sqlx::query_as!(
//             DbCostModel,
//             r#"
//             SELECT deployment, model, variables
//             FROM "CostModels"
//             WHERE deployment != 'global'
//             ORDER BY deployment ASC
//             "#
//         )
//         .fetch_all(pool)
//         .await?
//     } else {
//         sqlx::query_as!(
//             DbCostModel,
//             r#"
//             SELECT deployment, model, variables
//             FROM "CostModels"
//             WHERE deployment = ANY($1)
//             AND deployment != 'global'
//             ORDER BY deployment ASC
//             "#,
//             &hex_ids
//         )
//         .fetch_all(pool)
//         .await?
//     }
//     .into_iter()
//     .map(CostModel::try_from)
//     .collect::<Result<Vec<_>, _>>()?;

//     let deployments_with_models = models
//         .iter()
//         .map(|model| &model.deployment)
//         .collect::<HashSet<_>>();

//     let deployments_without_models = deployments
//         .iter()
//         .filter(|deployment| !deployments_with_models.contains(deployment))
//         .collect::<HashSet<_>>();

//     // Query the global cost model
//     if let Some(global_model) = global_cost_model(pool).await? {
//         // For all deployments that have a cost model, merge it with the global one
//         models = models
//             .into_iter()
//             .map(|model| merge_global(model, &global_model))
//             .collect();

//         // Inject a cost model for all deployments that don't have one
//         models = models
//             .into_iter()
//             .chain(
//                 deployments_without_models
//                     .into_iter()
//                     .map(|deployment| CostModel {
//                         deployment: deployment.to_owned(),
//                         model: global_model.model.clone(),
//                         variables: global_model.variables.clone(),
//                     }),
//             )
//             .collect();
//     }

//     Ok(models)
// }

// /// Make database query for a cost model indexed by deployment id
// pub async fn cost_model(
//     pool: &PgPool,
//     deployment: &DeploymentId,
// ) -> Result<Option<CostModel>, anyhow::Error> {
//     let model = sqlx::query_as!(
//         DbCostModel,
//         r#"
//         SELECT deployment, model, variables
//         FROM "CostModels"
//         WHERE deployment = $1
//         AND deployment != 'global'
//         "#,
//         format!("{:#x}", deployment),
//     )
//     .fetch_optional(pool)
//     .await?
//     .map(CostModel::try_from)
//     .transpose()?;

//     let global_model = global_cost_model(pool).await?;

//     Ok(match (model, global_model) {
//         // If we have no global model, return whatever we can find for the deployment
//         (None, None) => None,
//         (Some(model), None) => Some(model),

//         // If we have a cost model and a global cost model, merge them
//         (Some(model), Some(global_model)) => Some(merge_global(model, &global_model)),

//         // If we have only a global model, use that
//         (None, Some(global_model)) => Some(CostModel {
//             deployment: deployment.to_owned(),
//             model: global_model.model,
//             variables: global_model.variables,
//         }),
//     })
// }

// /// Query global cost model
// async fn global_cost_model(pool: &PgPool) -> Result<Option<DbCostModel>, anyhow::Error> {
//     sqlx::query_as!(
//         DbCostModel,
//         r#"
//         SELECT deployment, model, variables
//         FROM "CostModels"
//         WHERE deployment = $1
//         "#,
//         "global"
//     )
//     .fetch_optional(pool)
//     .await
//     .map_err(Into::into)
// }

// fn merge_global(model: CostModel, global_model: &DbCostModel) -> CostModel {
//     CostModel {
//         deployment: model.deployment,
//         model: model.model.clone().or(global_model.model.clone()),
//         variables: model.variables.clone().or(global_model.variables.clone()),
//     }
// }

// #[cfg(test)]
// mod test {

//     use std::str::FromStr;

//     use sqlx::PgPool;

//     use super::*;

//     async fn setup_cost_models_table(pool: &PgPool) {
//         sqlx::query!(
//             r#"
//             CREATE TABLE "CostModels"(
//                 id INT,
//                 deployment VARCHAR NOT NULL,
//                 model TEXT,
//                 variables JSONB,
//                 PRIMARY KEY( deployment )
//             );
//             "#,
//         )
//         .execute(pool)
//         .await
//         .expect("Create test instance in db");
//     }

//     async fn add_cost_models(pool: &PgPool, models: Vec<DbCostModel>) {
//         for model in models {
//             sqlx::query!(
//                 r#"
//                 INSERT INTO "CostModels" (deployment, model)
//                 VALUES ($1, $2);
//                 "#,
//                 model.deployment,
//                 model.model,
//             )
//             .execute(pool)
//             .await
//             .expect("Create test instance in db");
//         }
//     }

//     fn to_db_models(models: Vec<CostModel>) -> Vec<DbCostModel> {
//         models.into_iter().map(DbCostModel::from).collect()
//     }

//     fn global_cost_model() -> DbCostModel {
//         DbCostModel {
//             deployment: "global".to_string(),
//             model: Some("default => 0.00001;".to_string()),
//             variables: None,
//         }
//     }

//     fn test_data() -> Vec<CostModel> {
//         vec![
//             CostModel {
//                 deployment: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
//                     .parse()
//                     .unwrap(),
//                 model: None,
//                 variables: None,
//             },
//             CostModel {
//                 deployment: "0xbd499f7673ca32ef4a642207a8bebdd0fb03888cf2678b298438e3a1ae5206ea"
//                     .parse()
//                     .unwrap(),
//                 model: Some("default => 0.00025;".to_string()),
//                 variables: None,
//             },
//             CostModel {
//                 deployment: "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
//                     .parse()
//                     .unwrap(),
//                 model: Some("default => 0.00012;".to_string()),
//                 variables: None,
//             },
//         ]
//     }

//     #[sqlx::test]
//     async fn success_cost_models(pool: PgPool) {
//         let test_models = test_data();
//         let test_deployments = test_models
//             .iter()
//             .map(|model| model.deployment)
//             .collect::<HashSet<_>>();

//         setup_cost_models_table(&pool).await;
//         add_cost_models(&pool, to_db_models(test_models.clone())).await;

//         // First test: query without deployment filter
//         let models = cost_models(&pool, &[])
//             .await
//             .expect("cost models query without deployment filter");

//         // We expect as many models as we have in the test data
//         assert_eq!(models.len(), test_models.len());

//         // We expect models for all test deployments to be present and
//         // identical to the test data
//         for test_deployment in test_deployments.iter() {
//             let test_model = test_models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for test deployment in test data");

//             let model = models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for test deployment in query result");

//             assert_eq!(test_model.model, model.model);
//         }

//         // Second test: query with a deployment filter
//         let sample_deployments = vec![
//             test_models.get(0).unwrap().deployment,
//             test_models.get(1).unwrap().deployment,
//         ];
//         let models = cost_models(&pool, &sample_deployments)
//             .await
//             .expect("cost models query with deployment filter");

//         // Expect two cost mdoels to be returned
//         assert_eq!(models.len(), sample_deployments.len());

//         // Expect both returned deployments to be identical to the test data
//         for test_deployment in sample_deployments.iter() {
//             let test_model = test_models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for test deployment in test data");

//             let model = models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for test deployment in query result");

//             assert_eq!(test_model.model, model.model);
//         }
//     }

//     #[sqlx::test]
//     async fn global_fallback_cost_models(pool: PgPool) {
//         let test_models = test_data();
//         let test_deployments = test_models
//             .iter()
//             .map(|model| model.deployment)
//             .collect::<HashSet<_>>();
//         let global_model = global_cost_model();

//         setup_cost_models_table(&pool).await;
//         add_cost_models(&pool, to_db_models(test_models.clone())).await;
//         add_cost_models(&pool, vec![global_model.clone()]).await;

//         // First test: fetch cost models without filtering by deployment
//         let models = cost_models(&pool, &[])
//             .await
//             .expect("cost models query without deployments filter");

//         // Since we've defined 3 cost models and we did not provide a filter, we
//         // expect all of them to be returned except for the global cost model
//         assert_eq!(models.len(), test_models.len());

//         // Expect all test deployments to be present in the query result
//         for test_deployment in test_deployments.iter() {
//             let test_model = test_models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for deployment in test data");

//             let model = models
//                 .iter()
//                 .find(|model| &model.deployment == test_deployment)
//                 .expect("finding cost model for deployment in query result");

//             if test_model.model.is_some() {
//                 // If the test model has a model definition, we expect that to be returned
//                 assert_eq!(model.model, test_model.model);
//             } else {
//                 // If the test model has no model definition, we expect the global
//                 // model definition to be returned
//                 assert_eq!(model.model, global_model.model);
//             }
//         }

//         // Second test: fetch cost models, filtering by the first two deployment IDs
//         let sample_deployments = vec![
//             test_models.get(0).unwrap().deployment,
//             test_models.get(1).unwrap().deployment,
//         ];
//         let models = dbg!(cost_models(&pool, &sample_deployments).await)
//             .expect("cost models query with deployments filter");

//         // We've filtered by two deployment IDs and are expecting two cost models to be returned
//         assert_eq!(models.len(), sample_deployments.len());

//         for test_deployment in sample_deployments {
//             let test_model = test_models
//                 .iter()
//                 .find(|model| model.deployment == test_deployment)
//                 .expect("finding cost model for deployment in test data");

//             let model = models
//                 .iter()
//                 .find(|model| model.deployment == test_deployment)
//                 .expect("finding cost model for deployment in query result");

//             if test_model.model.is_some() {
//                 // If the test model has a model definition, we expect that to be returned
//                 assert_eq!(model.model, test_model.model);
//             } else {
//                 // If the test model has no model definition, we expect the global
//                 // model definition to be returned
//                 assert_eq!(model.model, global_model.model);
//             }
//         }

//         // Third test: query for missing cost model
//         let missing_deployment =
//             DeploymentId::from_str("Qmaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
//         let models = cost_models(&pool, &[missing_deployment])
//             .await
//             .expect("cost models query for missing deployment");

//         // The deployment may be missing in the database but we have a global model
//         // and expect that to be returned, with the missing deployment ID
//         let missing_model = models
//             .iter()
//             .find(|m| m.deployment == missing_deployment)
//             .expect("finding missing deployment");
//         assert_eq!(missing_model.model, global_model.model);
//     }

//     #[sqlx::test]
//     async fn success_cost_model(pool: PgPool) {
//         setup_cost_models_table(&pool).await;
//         add_cost_models(&pool, to_db_models(test_data())).await;

//         let deployment_id_from_bytes = DeploymentId(
//             "0xbd499f7673ca32ef4a642207a8bebdd0fb03888cf2678b298438e3a1ae5206ea"
//                 .parse()
//                 .unwrap(),
//         );
//         let deployment_id_from_hash =
//             DeploymentId::from_str("Qmb5Ysp5oCUXhLA8NmxmYKDAX2nCMnh7Vvb5uffb9n5vss").unwrap();

//         assert_eq!(deployment_id_from_bytes, deployment_id_from_hash);

//         let model = cost_model(&pool, &deployment_id_from_bytes)
//             .await
//             .expect("cost model query")
//             .expect("cost model for deployment");

//         assert_eq!(model.deployment, deployment_id_from_hash);
//         assert_eq!(model.model, Some("default => 0.00025;".to_string()));
//     }

//     #[sqlx::test]
//     async fn global_fallback_cost_model(pool: PgPool) {
//         let test_models = test_data();
//         let global_model = global_cost_model();

//         setup_cost_models_table(&pool).await;
//         add_cost_models(&pool, to_db_models(test_models.clone())).await;
//         add_cost_models(&pool, vec![global_model.clone()]).await;

//         // Test that the behavior is correct for existing deployments
//         for test_model in test_models {
//             let model = cost_model(&pool, &test_model.deployment)
//                 .await
//                 .expect("cost model query")
//                 .expect("global cost model fallback");

//             assert_eq!(model.deployment, test_model.deployment);

//             if test_model.model.is_some() {
//                 // If the test model has a model definition, we expect that to be returned
//                 assert_eq!(model.model, test_model.model);
//             } else {
//                 // If the test model has no model definition, we expect the global
//                 // model definition to be returned
//                 assert_eq!(model.model, global_model.model);
//             }
//         }

//         // Test that querying a non-existing deployment returns the default cost model
//         let missing_deployment =
//             DeploymentId::from_str("Qmnononononononononononononononononononononono").unwrap();
//         let model = cost_model(&pool, &missing_deployment)
//             .await
//             .expect("cost model query")
//             .expect("global cost model fallback");
//         assert_eq!(model.deployment, missing_deployment);
//         assert_eq!(model.model, global_model.model);
//     }
// }
