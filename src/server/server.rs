use crate::auth::permission::PermissionDataLoader;
use crate::auth::permission::PermissionLoader;
use crate::auth::principal::Principal;
use crate::auth::AuthState;
use crate::graphql::artifact_version::ArtifactVersionDataLoader;
use crate::graphql::artifact_version::ArtifactVersionLoader;
use crate::graphql::project::ProjectDataLoader;
use crate::graphql::project::ProjectLoader;
use crate::storage::artifact::Storage;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_trait::async_trait;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use std::convert::Infallible;
use std::sync::Arc;
use tracing::error;

#[derive(Clone)]
pub struct ServerState {
    pub storage: Storage,
    pub auth_state: AuthState,
}

impl ServerState {
    pub fn new_project_loader(
        &self,
        principal: &Principal,
        permission_loader: &PermissionDataLoader,
    ) -> ProjectDataLoader {
        Arc::new(DataLoader::with_cache(
            ProjectLoader {
                principal: principal.clone(),
                permission_loader: permission_loader.clone(),
                storage: self.storage.clone(),
            },
            tokio::spawn,
            HashMapCache::default(),
        ))
    }

    pub fn new_permission_loader(&self) -> PermissionDataLoader {
        Arc::new(DataLoader::with_cache(
            PermissionLoader {
                storage: self.storage.clone(),
            },
            tokio::spawn,
            HashMapCache::default(),
        ))
    }

    pub fn new_artifact_version_loader(
        &self,
        principal: &Principal,
        permission_loader: &PermissionDataLoader,
    ) -> ArtifactVersionDataLoader {
        Arc::new(DataLoader::with_cache(
            ArtifactVersionLoader {
                principal: principal.clone(),
                permission_loader: permission_loader.clone(),
                storage: self.storage.clone(),
            },
            tokio::spawn,
            HashMapCache::default(),
        ))
    }
}

impl FromRef<ServerState> for AuthState {
    fn from_ref(input: &ServerState) -> Self {
        input.auth_state.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ServerState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to
// turn them into `Result<_, AppError>`. That way you don't need to do that
// manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let e = err.into();
        error!("{}", e);
        Self(e)
    }
}
