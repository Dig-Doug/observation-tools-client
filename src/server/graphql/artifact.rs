use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
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
use futures_util::future::join_all;
use itertools::Itertools;
use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifacts::SeriesId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Artifact {
    pub id: AbsoluteArtifactId,
}

#[Object]
impl Artifact {
    pub async fn id(&self) -> async_graphql::Result<ID> {
        todo!("Implement Artifact::id")
    }
}

pub type ArtifactDataLoader = Arc<DataLoader<ArtifactLoader, HashMapCache>>;

pub struct ArtifactLoader {
    pub principal: Principal,
    pub permission_loader: PermissionDataLoader,
    pub storage: ArtifactStorage,
}

impl Loader<AbsoluteArtifactId> for ArtifactLoader {
    type Value = Result<Artifact, LoaderError>;
    type Error = LoaderError;

    async fn load(
        &self,
        keys: &[AbsoluteArtifactId],
    ) -> Result<HashMap<AbsoluteArtifactId, Self::Value>, Self::Error> {
        let accessible_ids = self
            .permission_loader
            .load_many(Permission::from_ids(
                self.principal.clone(),
                keys.to_vec(),
                Operation::Read,
            ))
            .await
            .map_err(|e| LoaderError::Error { message: e })?;
        let results: Vec<_> = join_all(
            accessible_ids
                .into_iter()
                .map(|(permission, accessible)| self.load_artifact(permission, accessible)),
        )
        .await;
        Ok(results.into_iter().collect())
    }
}

type LastArtifactStateBySeries = HashMap<Option<SeriesId>, Artifact>;

impl ArtifactLoader {
    async fn load_artifact(
        &self,
        permission: Permission<AbsoluteArtifactId>,
        access_result: AccessResult<AbsoluteArtifactId>,
    ) -> (AbsoluteArtifactId, Result<Artifact, LoaderError>) {
        let artifact_id = permission.resource_id.clone();
        if !access_result.allow {
            return (artifact_id, Err(permission.into()));
        }

        todo!("Impl")
    }

    async fn process_versions(
        &self,
        artifact_id: &AbsoluteArtifactId,
    ) -> Result<Artifact, anyhow::Error> {
        // TODO(doug): Is it worth streaming versions here, e.g. to lower memory usage?
        // #optimization
        let versions = self
            .storage
            .read_versions_for_artifact(&artifact_id)
            .await?;

        let versions = versions
            .into_iter()
            .filter(|version| {
                // If series point is set, keep versions where series_id is null or series id
                // match and series value is less than or equal to the point
                todo!("i")
            })
            .sorted_by(|a, b| {
                // Sort by series_id (nulls first), series_value (nulls first), creation time,
                // version_id
                todo!("i")
            })
            .try_collect()?;

        let last_artifact_states_by_series = self.merge_versions(versions);

        todo!("Impl")
    }

    fn merge_versions(&self, versions: Vec<ArtifactVersionRow>) -> LastArtifactStateBySeries {
        let mut last_artifact_states_by_series: LastArtifactStateBySeries = HashMap::new();
        for version in versions {
            let series_id = version.series_point.as_ref().map(|p| p.series_id.clone());
            let last_state = last_artifact_states_by_series
                .remove(&series_id)
                .or_else(|| last_artifact_states_by_series.get(&None).cloned());
            let artifact = self.merge_version(last_state.clone(), version);
            last_artifact_states_by_series.insert(series_id, artifact);
        }
        last_artifact_states_by_series
    }

    fn merge_version(&self, last_state: Option<Artifact>, version: ArtifactVersionRow) -> Artifact {
        todo!("Impl")
    }

    fn pick_series_to_return(
        &self,
        id: &AbsoluteArtifactId,
        last_artifact_states_by_series: LastArtifactStateBySeries,
    ) -> Option<SeriesId> {
        if last_artifact_states_by_series.contains_key(&None) {
            None
        } else {
            id.series_context
                .series_point()
                .map(|series_point| series_point.series_id.clone())
        }
    }
}
