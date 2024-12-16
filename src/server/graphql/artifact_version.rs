use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::util::calculate_start_and_length;
use crate::graphql::LoaderError;
use crate::storage::artifact::Storage;
use crate::storage::ArtifactVersionRow;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Object;
use async_graphql::ID;
use async_graphql::{connection, Context, Interface};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use observation_tools_common::artifact::{AbsoluteArtifactId, ArtifactVersionId};
use std::collections::HashMap;
use std::sync::Arc;
use crate::graphql::diff::get_text_content;

#[derive(Clone, Debug)]
pub struct ArtifactVersion {
    pub row: ArtifactVersionRow,
}

#[Object]
impl ArtifactVersion {
    pub async fn id(&self) -> async_graphql::Result<ID> {
        Ok(self.row.global_id().into())
    }

    pub async fn json(&self) -> async_graphql::Result<String> {
        Ok(serde_json::to_string(&self.row)?)
    }

    pub async fn client_creation_time(&self) -> DateTime<Utc> {
        self.row.version_data.client_creation_time
    }

    pub async fn name(&self) -> String {
        self.row.version_data.user_metadata.name.clone()
    }

    pub async fn payload(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        let storage = ctx.data::<Storage>()?;
        let payload = storage
            .read_artifact_version_payload(&self.row.absolute_id())
            .await?;
        Ok(payload.map(|p| get_text_content(p)).transpose()?)
    }

    pub async fn children(
        &self,
        ctx: &Context<'_>,
        direct_descendants_only: bool,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> async_graphql::Result<
        Connection<i32, ArtifactVersionOrArtifactVersionError, EmptyFields, EmptyFields>,
    > {
        let connection = connection::query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let (first_index, result_count) = calculate_start_and_length(
                    after,
                    before,
                    first.map(|v| v as i32),
                    last.map(|v| v as i32),
                    10,
                )?;

                let storage = ctx.data::<Storage>()?;
                let ids = storage
                    .get_child_artifacts(
                        &self.row,
                        direct_descendants_only,
                        first_index,
                        // Fetch one extra result to see if there is another page
                        result_count + 1,
                    )
                    .await?;

                create_artifact_version_connection(ctx, ids, first_index, result_count).await
            },
        )
        .await?;
        Ok(connection)
    }
}

#[derive(Interface, Clone, Debug)]
#[graphql(field(name = "id", ty = "ID"))]
pub enum ArtifactVersionOrArtifactVersionError {
    ArtifactVersion(ArtifactVersion),
    Error(ArtifactVersionError),
}

#[derive(Clone, Debug)]
pub struct ArtifactVersionError {
    pub id: ID,
    pub message: String,
}

#[Object]
impl ArtifactVersionError {
    pub async fn id(&self) -> ID {
        self.id.clone()
    }

    pub async fn message(&self) -> String {
        self.message.clone()
    }
}

pub type ArtifactVersionDataLoader = Arc<DataLoader<ArtifactVersionLoader, HashMapCache>>;

pub struct ArtifactVersionLoader {
    pub principal: Principal,
    pub permission_loader: PermissionDataLoader,
    pub storage: Storage,
}

impl Loader<AbsoluteArtifactVersionId> for ArtifactVersionLoader {
    type Value = Result<ArtifactVersion, LoaderError>;
    type Error = LoaderError;

    #[tracing::instrument(skip_all)]
    async fn load(
        &self,
        keys: &[AbsoluteArtifactVersionId],
    ) -> Result<HashMap<AbsoluteArtifactVersionId, Self::Value>, Self::Error> {
        // Permissions are at the artifact level, so group versions to fetch by artifact
        let mut versions_by_artifact: HashMap<AbsoluteArtifactId, Vec<AbsoluteArtifactVersionId>> =
            keys.iter()
                .cloned()
                .map(|id| (id.clone().into(), id))
                .into_group_map();
        let access_results = self
            .permission_loader
            .load_many(Permission::from_ids(
                self.principal.clone(),
                versions_by_artifact.keys().cloned().collect(),
                Operation::Read,
            ))
            .await
            .map_err(|e| LoaderError::Error { message: e })?;

        // Ungroup the versions so we have a map of version -> permission
        let version_permissions: HashMap<
            AbsoluteArtifactVersionId,
            &AccessResult<AbsoluteArtifactId>,
        > = access_results
            .iter()
            .flat_map(|(permission, access_result)| {
                versions_by_artifact
                    .remove(&permission.resource_id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(move |version_id| (version_id, access_result))
            })
            .collect();

        // Fetch the accessible versions from storage
        let versions_to_fetch: Vec<AbsoluteArtifactVersionId> = version_permissions
            .iter()
            .filter(|(version_id, access_result)| access_result.allow)
            .map(|(version_id, _)| version_id.clone())
            .collect();
        let versions = self
            .storage
            .read_artifact_versions(&versions_to_fetch)
            .await?;

        // Convert the storage rows into graphql objects
        let mut results: HashMap<AbsoluteArtifactVersionId, Self::Value> = versions
            .into_iter()
            .map(|(k, v)| {
                let v = v
                    .map(|row| ArtifactVersion { row })
                    .map_err(LoaderError::from);
                (k, v)
            })
            .collect();

        // Add errors for any versions that we could not access
        version_permissions
            .into_iter()
            .filter(|(version_id, access_result)| !access_result.allow)
            .for_each(|(version_id, access_result)| {
                results.insert(
                    version_id.clone(),
                    Err(access_result.clone().permission.into()),
                );
            });
        Ok(results)
    }
}

pub async fn create_artifact_version_connection(
    ctx: &Context<'_>,
    ids: Vec<AbsoluteArtifactVersionId>,
    first_index: usize,
    result_count: usize,
) -> async_graphql::Result<
    Connection<i32, ArtifactVersionOrArtifactVersionError, EmptyFields, EmptyFields>,
> {
    let artifact_version_loader = ctx.data::<ArtifactVersionDataLoader>()?;
    let mut versions = artifact_version_loader.load_many(ids.clone()).await?;

    let has_previous_page = first_index > 0;
    let has_next_page = versions.len() > result_count;
    let mut connection = Connection::new(has_previous_page, has_next_page);
    for (index, run_id) in ids.into_iter().take(result_count).enumerate() {
        let edge = versions
            .remove(&run_id)
            .map(|project_or_error| match project_or_error {
                Ok(project) => ArtifactVersionOrArtifactVersionError::ArtifactVersion(project),
                Err(err) => ArtifactVersionOrArtifactVersionError::Error(ArtifactVersionError {
                    id: run_id.clone().into(),
                    message: err.to_string(),
                }),
            })
            .unwrap_or_else(|| {
                ArtifactVersionOrArtifactVersionError::Error(ArtifactVersionError {
                    id: run_id.into(),
                    message: "Artifact version not found".to_string(),
                })
            });
        connection
            .edges
            .push(Edge::new((first_index + index) as i32, edge));
    }
    Ok::<_, async_graphql::Error>(connection)
}
