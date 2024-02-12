use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;

use file_exchange::manifest::{
    Bundle, BundleManifest, FileManifest, FileManifestMeta, FileMetaInfo,
};

use super::ServerContext;

/* Manifest types with GraphQL type derivation */

#[derive(Clone, Debug, SimpleObject)]
pub struct GraphQlBundleManifest {
    pub files: Vec<GraphQlFileMetaInfo>,
    pub file_type: String,
    pub spec_version: String,
    pub description: String,
    pub chain_id: String,
}

impl From<BundleManifest> for GraphQlBundleManifest {
    fn from(manifest: BundleManifest) -> Self {
        Self {
            files: manifest
                .files
                .into_iter()
                .map(GraphQlFileMetaInfo::from)
                .collect(),
            file_type: manifest.file_type,
            spec_version: manifest.spec_version,
            description: manifest.description,
            chain_id: manifest.chain_id,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct GraphQlBundle {
    pub ipfs_hash: String,
    //TODO: make local path available for admin only
    pub manifest: GraphQlBundleManifest,
    pub file_manifests: Vec<GraphQlFileManifestMeta>,
}

impl From<Bundle> for GraphQlBundle {
    fn from(bundle: Bundle) -> Self {
        Self {
            ipfs_hash: bundle.ipfs_hash,
            manifest: GraphQlBundleManifest::from(bundle.manifest),
            file_manifests: bundle
                .file_manifests
                .into_iter()
                .map(GraphQlFileManifestMeta::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct GraphQlFileManifestMeta {
    pub meta_info: GraphQlFileMetaInfo,
    pub file_manifest: GraphQlFileManifest,
}

impl From<FileManifestMeta> for GraphQlFileManifestMeta {
    fn from(meta: FileManifestMeta) -> Self {
        Self {
            meta_info: GraphQlFileMetaInfo::from(meta.meta_info),
            file_manifest: GraphQlFileManifest::from(meta.file_manifest),
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct GraphQlFileMetaInfo {
    pub name: String,
    pub hash: String,
}

impl From<FileMetaInfo> for GraphQlFileMetaInfo {
    fn from(manifest: FileMetaInfo) -> Self {
        Self {
            name: manifest.name,
            hash: manifest.hash,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphQlFileManifest {
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub chunk_hashes: Vec<String>,
}

impl From<FileManifest> for GraphQlFileManifest {
    fn from(manifest: FileManifest) -> Self {
        Self {
            total_bytes: manifest.total_bytes,
            chunk_size: manifest.chunk_size,
            chunk_hashes: manifest.chunk_hashes,
        }
    }
}

#[derive(Default)]
pub struct StatusQuery;

#[Object]
impl StatusQuery {
    async fn files(
        &self,
        ctx: &Context<'_>,
        deployments: Option<Vec<String>>,
    ) -> Result<Vec<GraphQlFileManifest>, anyhow::Error> {
        let bundles: Vec<Bundle> = ctx
            .data_unchecked::<ServerContext>()
            .state
            .lock()
            .await
            .bundles
            .values()
            .cloned()
            .collect();
        let file_metas: Vec<FileManifestMeta> = bundles
            .iter()
            .flat_map(|b| b.file_manifests.clone())
            .collect();

        if deployments.is_none() {
            return Ok(file_metas
                .iter()
                .map(|m| GraphQlFileManifest::from(m.file_manifest.clone()))
                .collect::<Vec<GraphQlFileManifest>>());
        };
        let ids = deployments.unwrap();
        Ok(file_metas
            .iter()
            .filter(|m| ids.contains(&m.meta_info.hash))
            .map(|m| m.file_manifest.clone())
            .map(GraphQlFileManifest::from)
            .collect())
    }

    async fn file(
        &self,
        ctx: &Context<'_>,
        deployment: String,
    ) -> Result<Option<GraphQlFileManifest>, anyhow::Error> {
        let bundles: Vec<Bundle> = ctx
            .data_unchecked::<ServerContext>()
            .state
            .lock()
            .await
            .bundles
            .values()
            .cloned()
            .collect();
        let file_metas: Vec<FileManifestMeta> = bundles
            .iter()
            .flat_map(|b| b.file_manifests.clone())
            .collect();
        let manifest_graphql = file_metas
            .iter()
            .find(|m| m.meta_info.hash == deployment)
            .map(|m| m.file_manifest.clone())
            .map(GraphQlFileManifest::from);

        Ok(manifest_graphql)
    }

    async fn bundles(
        &self,
        ctx: &Context<'_>,
        deployments: Option<Vec<String>>,
    ) -> Result<Vec<GraphQlBundle>, anyhow::Error> {
        tracing::info!("receive bundles request");
        let all_bundles = &ctx
            .data_unchecked::<ServerContext>()
            .state
            .lock()
            .await
            .bundles.clone();

        tracing::info!(bundles = tracing::field::debug(&all_bundles), "all bundles");
        let bundles = if deployments.is_none() {
            tracing::info!(
                bundles = tracing::field::debug(&all_bundles),
                "no deployment filter"
            );
            all_bundles
                .values()
                .cloned()
                .map(GraphQlBundle::from)
                .collect()
        } else {
            let ids = deployments.unwrap();
            ids.iter()
                .filter_map(|key| all_bundles.get(key))
                .cloned()
                .map(GraphQlBundle::from)
                .collect()
        };
        tracing::info!(bundles = tracing::field::debug(&bundles), "queried bundles");
        Ok(bundles)
    }

    async fn status(
        &self,
        ctx: &Context<'_>,
        deployment: String,
    ) -> Result<Option<GraphQlBundle>, anyhow::Error> {
        // let deployment_id = DeploymentId::from_str(&deployment)?;
        let bundle: Option<Bundle> = ctx
            .data_unchecked::<ServerContext>()
            .state
            .lock()
            .await
            .bundles
            .get(&deployment)
            .cloned();

        Ok(bundle.map(GraphQlBundle::from))
    }
}

pub type StatusSchema = Schema<StatusQuery, EmptyMutation, EmptySubscription>;

pub async fn build_schema() -> StatusSchema {
    Schema::build(StatusQuery, EmptyMutation, EmptySubscription).finish()
}

pub async fn status(State(context): State<ServerContext>, req: GraphQLRequest) -> GraphQLResponse {
    context
        .state
        .lock()
        .await
        .status_schema
        .execute(req.into_inner().data(context.clone()))
        .await
        .into()
}
