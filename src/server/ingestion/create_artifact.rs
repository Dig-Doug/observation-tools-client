use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::auth::AuthState;
use crate::server::AppError;
use crate::server::ServerState;
use crate::storage::ArtifactStorage;
use crate::storage::ArtifactVersion;
use anyhow::anyhow;
use axum::async_trait;
use axum::extract::multipart::Field;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::extract::Multipart;
use axum::extract::State;
use axum::http::request::Parts;
use futures_util::TryStreamExt;
use observation_tools_common::create_artifact::CreateArtifactRequest;
use std::convert::Infallible;
use uuid::Uuid;

#[derive(Clone)]
pub struct CreateArtifactState {
    pub permission_loader: PermissionDataLoader,
    pub artifact_storage: ArtifactStorage,
    pub auth_state: AuthState,
}

#[async_trait]
impl<S> FromRequestParts<S> for CreateArtifactState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

impl FromRef<CreateArtifactState> for AuthState {
    fn from_ref(input: &CreateArtifactState) -> Self {
        Self::from_ref(&input.auth_state)
    }
}

#[axum::debug_handler]
pub async fn create_artifact(
    State(state): State<CreateArtifactState>,
    principal: Principal,
    mut multipart: Multipart,
) -> Result<(), AppError> {
    let request = read_and_validate_request(multipart.next_field().await?).await?;

    let allowed = state
        .permission_loader
        .load_one(Permission::new(
            principal,
            request.project_id.clone(),
            Operation::Write,
        ))
        .await
        .map_err(|e| anyhow!("Error loading permission: {}", e))?;

    if allowed.unwrap_or(AccessResult::Deny) != AccessResult::Allow {
        return Err(anyhow!("Not authorized to write to project"))?;
    }

    let version = ArtifactVersion {
        project_id: request.project_id,
        run_id: request.run_id,
        artifact_id: request.artifact_id,
        version_id: Uuid::new_v4(),
        version_data: request.payload,
        series_point: request.series_point,
    };

    let field = match multipart.next_field().await? {
        Some(field) => field,
        None => return Err(anyhow!("Missing `raw_data` field in multipart upload"))?,
    };

    let name = field.name().unwrap_or_default();
    if name != "raw_data" {
        return Err(anyhow!(
            "Second field in multipart upload must be `raw_data`, got `{}`",
            name
        ))?;
    }
    state
        .artifact_storage
        .write_artifact_version(version, field.into_stream())
        .await?;

    // TODO(doug): Setup permissions for run
    // TODO(doug): Notify new run
    // TODO(doug): Notify new artifact

    Ok(())
}

async fn read_and_validate_request(
    field: Option<Field<'_>>,
) -> Result<CreateArtifactRequest, anyhow::Error> {
    let field = match field {
        Some(field) => field,
        None => {
            return Err(anyhow::anyhow!(
                "Missing `{}` field in multipart upload",
                CreateArtifactRequest::HTTP_FIELD_NAME
            ))
        }
    };
    let name = field.name().unwrap_or_default();
    if name != CreateArtifactRequest::HTTP_FIELD_NAME {
        return Err(anyhow::anyhow!(
            "First field in multipart upload must be `{}`, got `{}`",
            CreateArtifactRequest::HTTP_FIELD_NAME,
            name
        ));
    }
    let data = field.bytes().await?;
    let unvalidated_request: CreateArtifactRequest = rmp_serde::from_read(data.as_ref())?;
    Ok(unvalidated_request.validate()?)
}
