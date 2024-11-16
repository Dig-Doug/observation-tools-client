use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionStorage;
use crate::auth::principal::Principal;
use crate::graphql::project::Project;
use crate::graphql::project::ProjectDataLoader;
use crate::storage::artifact::ArtifactStorage;
use crate::storage::project::ProjectData;
use crate::storage::project::ProjectRow;
use async_graphql::Context;
use async_graphql::Object;
use observation_tools_common::project::ProjectId;

#[derive(Default)]
pub struct CreateProjectMutation {}

#[Object]
impl CreateProjectMutation {
    async fn create_project(&self, ctx: &Context<'_>) -> async_graphql::Result<Project> {
        let principal = ctx.data::<Principal>()?;
        // TODO(doug): Add max # of projects?

        let storage = ctx.data::<ArtifactStorage>()?;
        let project_id = ProjectId::new();
        storage
            .create_project(ProjectRow {
                id: project_id.clone(),
                data: ProjectData {
                    name: "New project".to_string(),
                },
            })
            .await?;

        let permission_storage = ctx.data::<PermissionStorage>()?;
        permission_storage
            .create_permission(Permission {
                principal: principal.id(),
                resource_id: project_id.clone(),
                operation: Operation::Owner,
            })
            .await?;

        let project_loader = ctx.data::<ProjectDataLoader>()?;
        let project = project_loader
            .load_one(project_id)
            .await?
            .ok_or("Failed to get new project")?;
        Ok(project?)
    }
}
