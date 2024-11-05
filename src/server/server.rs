use crate::auth::permission::PermissionLoader;
use crate::auth::AuthState;
use crate::ingestion::create_artifact::CreateArtifactState;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {}

impl FromRef<ServerState> for CreateArtifactState {
    fn from_ref(input: &ServerState) -> Self {
        CreateArtifactState {
            permission_loader: Arc::new(DataLoader::with_cache(
                PermissionLoader {},
                tokio::spawn,
                HashMapCache::default(),
            )),
        }
    }
}

impl FromRef<ServerState> for AuthState {
    fn from_ref(input: &ServerState) -> Self {
        AuthState {}
    }
}
