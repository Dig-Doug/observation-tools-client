use crate::auth::permission::load_permissions_and_filter_ids;
use crate::auth::permission::Operation;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::graphql::artifact::ArtifactOrArtifactError;
use crate::graphql::artifact_version::ArtifactVersionDataLoader;
use crate::graphql::artifact_version::ArtifactVersionError;
use crate::graphql::artifact_version::ArtifactVersionOrArtifactVersionError;
use crate::graphql::util::calculate_start_and_length;
use crate::graphql::LoaderError;
use crate::storage::artifact::Storage;
use crate::storage::project::ProjectRow;
use crate::storage::ArtifactVersionRowOrError;
use async_graphql::connection;
use async_graphql::connection::Connection;
use async_graphql::connection::Edge;
use async_graphql::connection::EmptyFields;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use async_graphql::Context;
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

    pub async fn runs(
        &self,
        ctx: &Context<'_>,
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
                let run_ids = storage
                    .get_run_ids(
                        &self.row.id,
                        first_index,
                        // Fetch one extra result to see if there is another page
                        result_count + 1,
                    )
                    .await?;

                let artifact_version_loader = ctx.data::<ArtifactVersionDataLoader>()?;
                let mut projects = artifact_version_loader.load_many(run_ids.clone()).await?;

                let has_previous_page = first_index > 0;
                let has_next_page = projects.len() > result_count;
                let mut connection = Connection::new(has_previous_page, has_next_page);
                for (index, run_id) in run_ids.into_iter().take(result_count).enumerate() {
                    let edge = projects
                        .remove(&run_id)
                        .map(|project_or_error| match project_or_error {
                            Ok(project) => {
                                ArtifactVersionOrArtifactVersionError::ArtifactVersion(project)
                            }
                            Err(err) => {
                                ArtifactVersionOrArtifactVersionError::Error(ArtifactVersionError {
                                    id: run_id.clone().into(),
                                    message: err.to_string(),
                                })
                            }
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
            },
        )
        .await?;
        Ok(connection)
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
