use crate::auth::permission::load_permissions_and_filter_ids;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::LoaderError;
use crate::storage::artifact::Storage;
use crate::storage::project::ProjectRow;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Interface;
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

#[derive(Interface, Clone, Debug)]
#[graphql(field(name = "id", ty = "ID"))]
pub enum ProjectOrProjectError {
    Project(Project),
    Error(ProjectError),
}

#[derive(Clone, Debug)]
pub struct ProjectError {
    pub id: ID,
    pub message: String,
}

#[Object]
impl ProjectError {
    pub async fn id(&self) -> ID {
        self.id.clone()
    }

    pub async fn message(&self) -> String {
        self.message.clone()
    }
}

pub type ProjectDataLoader = Arc<DataLoader<ProjectLoader, HashMapCache>>;

pub struct ProjectLoader {
    pub principal: Principal,
    pub permission_loader: PermissionDataLoader,
    pub storage: Storage,
}

impl Loader<ProjectId> for ProjectLoader {
    type Value = Result<Project, LoaderError>;
    type Error = LoaderError;

    async fn load(
        &self,
        keys: &[ProjectId],
    ) -> Result<HashMap<ProjectId, Self::Value>, Self::Error> {
        let (accessible_projects, projects_to_fetch) =
            load_permissions_and_filter_ids(&self.permission_loader, &self.principal, keys).await?;
        let results = self.storage.read_projects(&projects_to_fetch).await?;
        let mut results: HashMap<ProjectId, Self::Value> = results
            .into_iter()
            .map(|(k, v)| {
                let v = v.map(|row| Project { row }).map_err(LoaderError::from);
                (k, v)
            })
            .collect();
        for (project_id, accessible) in accessible_projects.into_iter() {
            if accessible.allow {
                results.entry(project_id.clone()).or_insert_with(|| {
                    Err(LoaderError::ProjectNotFound {
                        project_id: project_id.into(),
                    })
                });
            } else {
                results.insert(project_id.clone(), Err(accessible.permission.into()));
            }
        }
        Ok(results)
    }
}
