use crate::auth::permission::Operation;
use crate::auth::permission::PermissionStorage;
use crate::auth::principal::Principal;
use crate::graphql::project::ProjectDataLoader;
use crate::graphql::project::ProjectError;
use crate::graphql::project::ProjectOrProjectError;
use crate::graphql::util::calculate_start_and_length;
use async_graphql::connection;
use async_graphql::connection::Connection;
use async_graphql::connection::Edge;
use async_graphql::connection::EmptyFields;
use async_graphql::Context;
use async_graphql::Object;
use observation_tools_common::project::ProjectId;

#[derive(Default)]
pub struct GetProjectsQuery {}

#[Object]
impl GetProjectsQuery {
    async fn get_projects(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> async_graphql::Result<Connection<i32, ProjectOrProjectError, EmptyFields, EmptyFields>>
    {
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

                let principal = ctx.data::<Principal>()?;
                let permission_storage = ctx.data::<PermissionStorage>()?;
                let project_ids = permission_storage
                    .get_resources::<ProjectId>(
                        principal,
                        Operation::Read,
                        first_index,
                        // Fetch one extra result to see if there is another page
                        result_count + 1,
                    )
                    .await?;

                let project_loader = ctx.data::<ProjectDataLoader>()?;
                let mut projects = project_loader.load_many(project_ids.clone()).await?;

                let has_previous_page = first_index > 0;
                let has_next_page = projects.len() > result_count;
                let mut connection = Connection::new(has_previous_page, has_next_page);
                for (index, project_id) in project_ids.into_iter().enumerate() {
                    let edge = projects
                        .remove(&project_id)
                        .map(|project_or_error| match project_or_error {
                            Ok(project) => ProjectOrProjectError::Project(project),
                            Err(err) => ProjectOrProjectError::Error(ProjectError {
                                id: project_id.clone().into(),
                                message: err.to_string(),
                            }),
                        })
                        .unwrap_or_else(|| {
                            ProjectOrProjectError::Error(ProjectError {
                                id: project_id.into(),
                                message: "Project not found".to_string(),
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
