use crate::auth::permission::PermissionLoader;
use crate::auth::AuthState;
use crate::ingestion::create_artifact::CreateArtifactState;
use crate::storage::ArtifactStorage;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use std::sync::Arc;
use tracing::error;

#[derive(Clone)]
pub struct ServerState {
    pub artifact_storage: ArtifactStorage,
    pub auth_state: AuthState,
}

impl FromRef<ServerState> for CreateArtifactState {
    fn from_ref(input: &ServerState) -> Self {
        CreateArtifactState {
            permission_loader: Arc::new(DataLoader::with_cache(
                PermissionLoader {},
                tokio::spawn,
                HashMapCache::default(),
            )),
            artifact_storage: input.artifact_storage.clone(),
            auth_state: input.auth_state.clone(),
        }
    }
}

impl FromRef<ServerState> for AuthState {
    fn from_ref(input: &ServerState) -> Self {
        AuthState::from_ref(&input.auth_state)
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
