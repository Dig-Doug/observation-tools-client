pub mod artifact;
mod create_project;
mod node;
pub mod project;

use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::ResourceId;
use crate::auth::principal::Principal;
use crate::graphql::create_project::CreateProjectMutation;
use crate::graphql::node::GetNodeQuery;
use crate::server::ServerState;
use async_graphql::http::playground_source;
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::MergedObject;
use async_graphql::Schema;
use async_graphql::ID;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use observation_tools_common::GlobalId;

#[derive(Default)]
struct QueryImpl;

#[derive(MergedObject, Default)]
struct Query(CreateProjectMutation, GetNodeQuery);

type ServerSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

pub async fn graphql_handler(
    State(server_state): State<ServerState>,
    principal: Principal,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let permission_loader = server_state.new_permission_loader();
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .data(principal.clone())
        .data(server_state.artifact_storage.clone())
        .data(server_state.permission_storage.clone())
        .data(server_state.new_project_loader(&principal, &permission_loader))
        .data(server_state.new_artifact_version_loader(&principal, &permission_loader))
        .data(permission_loader)
        .extension(async_graphql::extensions::Tracing)
        .finish();

    let req = req.into_inner();
    schema.execute(req).await.into()
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum LoaderError {
    #[error("Error loading data: {message}")]
    Error { message: String },
    #[error("Project not found: {project_id:?}")]
    ProjectNotFound { project_id: ID },
    #[error("ArtifactVersion not found: {artifact_version_id:?}")]
    ArtifactVersionNotFound { artifact_version_id: ID },
    #[error("Principal {principal:?} not authorized to {operation:?} on {resource_id:?}")]
    NotAuthorized {
        resource_id: ID,
        operation: Operation,
        principal: Principal,
    },
}

impl From<anyhow::Error> for LoaderError {
    fn from(err: anyhow::Error) -> Self {
        Self::Error {
            message: err.to_string(),
        }
    }
}

impl<T> From<Permission<T>> for LoaderError
where
    T: ResourceId + Into<GlobalId>,
{
    fn from(permission: Permission<T>) -> Self {
        Self::NotAuthorized {
            resource_id: permission.resource_id.into().into(),
            operation: permission.operation,
            principal: permission.principal,
        }
    }
}
