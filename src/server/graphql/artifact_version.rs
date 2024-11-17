use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::LoaderError;
use crate::storage::artifact::Storage;
use crate::storage::ArtifactVersionRow;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Object;
use async_graphql::ID;
use itertools::Itertools;
use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use std::collections::HashMap;
use std::sync::Arc;

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
