use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, routing::get, Router, Server};
use http::HeaderMap;
use tokio::sync::Mutex;

use crate::file_server::status::{GraphQlBundle, StatusQuery};
use crate::file_server::util::graphql_playground;
use crate::file_server::{FileServiceError, ServerContext};
use file_exchange::{
    errors::{Error, ServerError},
    manifest::{
        ipfs::IpfsClient, manifest_fetcher::read_bundle, validate_bundle_and_location, LocalBundle,
    },
};

#[derive(Clone)]
pub struct AdminState {
    pub client: IpfsClient,
    pub bundles: Arc<Mutex<HashMap<String, LocalBundle>>>,
    pub admin_auth_token: Option<String>,
    pub admin_schema: AdminSchema,
}

#[derive(Clone)]
pub struct AdminContext {
    pub state: Arc<AdminState>,
}

impl AdminContext {
    pub fn new(state: Arc<AdminState>) -> Self {
        Self { state }
    }
}

pub type AdminSchema = Schema<StatusQuery, StatusMutation, EmptySubscription>;

pub async fn build_schema() -> AdminSchema {
    Schema::build(StatusQuery, StatusMutation, EmptySubscription).finish()
}

fn get_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().map(|s| s.to_string()).ok())
}

// GraphQL handler to update status schema
//TODO: add mutation query fn for on-chain management?
async fn graphql_handler(
    State(context): State<AdminContext>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner().data(context.clone());
    if let Some(token) = get_token_from_headers(&headers) {
        req = req.data(token);
    }
    context.state.admin_schema.execute(req).await.into()
}

pub fn serve_admin(context: ServerContext) {
    tokio::spawn(async move {
        let admin_context = AdminContext::new(
            AdminState {
                client: context.state.client.clone(),
                bundles: context.state.bundles.clone(),
                admin_auth_token: context.state.admin_auth_token.clone(),
                admin_schema: build_schema().await,
            }
            .into(),
        );
        tracing::info!(address = %context.state.config.server.admin_host_and_port, "Serve admin metrics");

        let router = Router::new()
            .route("/admin", get(graphql_playground).post(graphql_handler))
            .with_state(admin_context);

        Server::bind(&context.state.config.server.admin_host_and_port)
            .serve(router.into_make_service())
            .await
            .expect("Failed to initialize admin server")
    });
}

/// Create an admin error response
pub fn admin_error_response(msg: &str) -> FileServiceError {
    FileServiceError::AdminError(Error::ServerError(ServerError::InvalidAuthentication(
        msg.to_string(),
    )))
}

#[derive(Default)]
pub struct StatusMutation;

#[Object]
impl StatusMutation {
    // Add a bundle
    async fn add_bundle(
        &self,
        ctx: &Context<'_>,
        deployment: String,
        location: String,
    ) -> Result<GraphQlBundle, anyhow::Error> {
        if ctx.data_opt::<String>()
            != ctx
                .data_unchecked::<AdminContext>()
                .state
                .admin_auth_token
                .as_ref()
        {
            return Err(anyhow::anyhow!(format!(
                "Failed to authenticate: {:#?} (admin: {:#?}",
                ctx.data_opt::<String>(),
                ctx.data_unchecked::<AdminContext>()
                    .state
                    .admin_auth_token
                    .as_ref()
            )));
        }
        let (hash, loc) = match validate_bundle_and_location(&deployment, &location) {
            Ok(s) => s,
            Err(e) => return Err(anyhow::anyhow!("Invalid input: {}", e.to_string())),
        };
        let bundle =
            match read_bundle(&ctx.data_unchecked::<AdminContext>().state.client, &hash).await {
                Ok(s) => s,
                Err(e) => return Err(anyhow::anyhow!(e.to_string(),)),
            };

        ctx.data_unchecked::<AdminContext>()
            .state
            .bundles
            .lock()
            .await
            .insert(
                bundle.ipfs_hash.clone(),
                LocalBundle {
                    bundle: bundle.clone(),
                    local_path: loc,
                },
            );

        Ok(GraphQlBundle::from(bundle))
    }

    // Add multiple bundles
    async fn add_bundles(
        &self,
        ctx: &Context<'_>,
        deployments: Vec<String>,
        locations: Vec<String>,
    ) -> Result<Vec<GraphQlBundle>, anyhow::Error> {
        if ctx.data_opt::<String>()
            != ctx
                .data_unchecked::<AdminContext>()
                .state
                .admin_auth_token
                .as_ref()
        {
            return Err(anyhow::anyhow!("Failed to authenticate"));
        }
        let client = ctx.data_unchecked::<AdminContext>().state.client.clone();
        let bundle_ref = ctx.data_unchecked::<AdminContext>().state.bundles.clone();
        let bundles = deployments
            .iter()
            .zip(locations)
            .map(|(deployment, location)| {
                let client = client.clone();
                let bundle_ref = bundle_ref.clone();

                async move {
                    tracing::debug!(deployment, location, "Adding bundle");

                    let (hash, loc) = validate_bundle_and_location(deployment, &location)
                        .map_err(|e| anyhow::anyhow!("Invalid input: {}", e))?;

                    let bundle = read_bundle(&client.clone(), &hash)
                        .await
                        .map_err(|e| anyhow::anyhow!("{}", e))?;

                    bundle_ref.clone().lock().await.insert(
                        bundle.ipfs_hash.clone(),
                        LocalBundle {
                            bundle: bundle.clone(),
                            local_path: loc,
                        },
                    );

                    Ok::<_, anyhow::Error>(GraphQlBundle::from(bundle))
                }
            })
            .collect::<Vec<_>>();

        // Since collect() gathers futures, we need to resolve them. You can use `try_join_all` for this.
        let resolved_bundles: Result<Vec<GraphQlBundle>, _> =
            futures::future::try_join_all(bundles).await;

        Ok(resolved_bundles.unwrap_or_default())
    }

    async fn remove_bundle(
        &self,
        ctx: &Context<'_>,
        deployment: String,
    ) -> Result<Option<GraphQlBundle>, anyhow::Error> {
        if ctx.data_opt::<String>()
            != ctx
                .data_unchecked::<AdminContext>()
                .state
                .admin_auth_token
                .as_ref()
        {
            return Err(anyhow::anyhow!("Failed to authenticate"));
        }

        let bundle = ctx
            .data_unchecked::<AdminContext>()
            .state
            .bundles
            .lock()
            .await
            .remove(&deployment)
            .map(|b| GraphQlBundle::from(b.bundle));

        Ok(bundle)
    }

    async fn remove_bundles(
        &self,
        ctx: &Context<'_>,
        deployments: Vec<String>,
    ) -> Result<Vec<GraphQlBundle>, anyhow::Error> {
        if ctx.data_opt::<String>()
            != ctx
                .data_unchecked::<AdminContext>()
                .state
                .admin_auth_token
                .as_ref()
        {
            return Err(anyhow::anyhow!("Failed to authenticate"));
        }

        let bundles = deployments
            .iter()
            .map(|deployment| async move {
                ctx.data_unchecked::<AdminContext>()
                    .state
                    .bundles
                    .lock()
                    .await
                    .remove(deployment)
                    .map(|b| GraphQlBundle::from(b.bundle))
                    .ok_or(anyhow::anyhow!(format!(
                        "Deployment not found: {}",
                        deployment
                    )))
            })
            .collect::<Vec<_>>();

        let removed_bundles: Result<Vec<GraphQlBundle>, _> =
            futures::future::try_join_all(bundles).await;

        removed_bundles
    }
}
