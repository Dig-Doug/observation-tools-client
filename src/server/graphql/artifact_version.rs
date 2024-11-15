use crate::auth::permission::load_permissions_and_filter_ids;
use crate::auth::permission::AccessResult;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::LoaderError;
use crate::storage::artifact::ArtifactStorage;
use crate::storage::ArtifactVersionRow;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Object;
use async_graphql::ID;
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
    pub storage: ArtifactStorage,
}

impl Loader<AbsoluteArtifactVersionId> for ArtifactVersionLoader {
    type Value = Result<ArtifactVersion, LoaderError>;
    type Error = LoaderError;

    async fn load(
        &self,
        keys: &[AbsoluteArtifactVersionId],
    ) -> Result<HashMap<AbsoluteArtifactVersionId, Self::Value>, Self::Error> {
        let (accessible_artifacts, artifacts_to_fetch) =
            load_permissions_and_filter_ids(&self.permission_loader, &self.principal, keys).await?;
        let results = self
            .storage
            .read_artifact_versions(&artifacts_to_fetch)
            .await?;
        let mut results: HashMap<AbsoluteArtifactVersionId, Self::Value> = results
            .into_iter()
            .map(|(k, v)| {
                let v = v
                    .map(|row| ArtifactVersion { row })
                    .map_err(LoaderError::from);
                (k, v)
            })
            .collect();
        for (permission, accessible) in accessible_artifacts.into_iter() {
            if accessible == AccessResult::Allow {
                results
                    .entry(permission.resource_id.clone())
                    .or_insert_with(|| {
                        Err(LoaderError::ArtifactVersionNotFound {
                            artifact_version_id: permission.resource_id.into(),
                        })
                    });
            } else {
                results.insert(permission.resource_id.clone(), Err(permission.into()));
            }
        }
        Ok(results)
    }
}
