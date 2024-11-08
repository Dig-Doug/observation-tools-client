use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::LoaderError;
use crate::storage::artifact::ArtifactStorage;
use crate::storage::project::ProjectRow;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Object;
use async_graphql::ID;
use observation_tools_common::project::ProjectId;
use observation_tools_common::GlobalId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Project {
    pub row: ProjectRow,
}

#[Object]
impl Project {
    pub async fn id(&self) -> async_graphql::Result<ID> {
        let global_id: GlobalId = self.row.id.clone().try_into()?;
        Ok(global_id.into())
    }
}

pub type ProjectDataLoader = Arc<DataLoader<ProjectLoader, HashMapCache>>;

pub struct ProjectLoader {
    pub principal: Principal,
    pub permission_loader: PermissionDataLoader,
    pub storage: ArtifactStorage,
}

impl Loader<ProjectId> for ProjectLoader {
    type Value = Result<Project, LoaderError>;
    type Error = LoaderError;

    async fn load(
        &self,
        keys: &[ProjectId],
    ) -> Result<HashMap<ProjectId, Self::Value>, Self::Error> {
        let accessible_projects = self
            .permission_loader
            .load_many(Permission::from_ids(
                self.principal.clone(),
                keys.to_vec(),
                Operation::Read,
            ))
            .await
            .map_err(|e| LoaderError::Error { message: e })?;
        let projects_to_fetch: Vec<ProjectId> = accessible_projects
            .iter()
            .filter(|(_, accessible)| **accessible == AccessResult::Allow)
            .map(|(permission, _)| permission.resource_id.clone())
            .collect();

        let results = self.storage.read_projects(projects_to_fetch).await?;
        let mut results: HashMap<ProjectId, Self::Value> = results
            .into_iter()
            .map(|(k, v)| {
                let v = v.map(|row| Project { row }).map_err(LoaderError::from);
                (k, v)
            })
            .collect();
        for (permission, accessible) in accessible_projects.into_iter() {
            if accessible == AccessResult::Allow {
                results
                    .entry(permission.resource_id.clone())
                    .or_insert_with(|| {
                        Err(LoaderError::ProjectNotFound {
                            project_id: permission.resource_id.into(),
                        })
                    });
            } else {
                results.insert(permission.resource_id.clone(), Err(permission.into()));
            }
        }
        Ok(results)
    }
}
