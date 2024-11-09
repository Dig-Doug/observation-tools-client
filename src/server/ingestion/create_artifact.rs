use crate::auth::permission::AccessResult;
use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::principal::Principal;
use crate::auth::AuthState;
use crate::graphql::project::ProjectDataLoader;
use crate::server::AppError;
use crate::server::ServerState;
use crate::storage::artifact::ArtifactStorage;
use crate::storage::ArtifactVersionRow;
use anyhow::anyhow;
use axum::async_trait;
use axum::extract::multipart::Field;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::extract::Multipart;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use futures_util::TryStreamExt;
use observation_tools_common::create_artifact::CreateArtifactRequest;
use uuid::Uuid;

#[derive(Clone)]
pub struct CreateArtifactState {
    pub permission_loader: PermissionDataLoader,
    pub artifact_storage: ArtifactStorage,
    pub auth_state: AuthState,
    pub project_loader: ProjectDataLoader,
}

#[async_trait]
impl<S> FromRequestParts<S> for CreateArtifactState
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let server_state = parts
            .extract_with_state::<ServerState, _>(state)
            .await
            .map_err(|_e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to extract auth state".to_string(),
                )
            })?;
        let principal = Principal::from_request_parts(parts, &server_state).await?;
        let permission_loader = server_state.new_permission_loader();
        let project_loader = server_state.new_project_loader(&principal, &permission_loader);
        Ok(CreateArtifactState {
            permission_loader,
            artifact_storage: server_state.artifact_storage.clone(),
            auth_state: server_state.auth_state.clone(),
            project_loader,
        })
    }
}

#[axum::debug_handler(state = ServerState)]
pub async fn create_artifact(
    state: CreateArtifactState,
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

    let version = ArtifactVersionRow {
        project_id: request.project_id,
        run_id: request.run_id,
        artifact_id: request.artifact_id,
        version_id: Uuid::new_v4(),
        version_data: request.payload,
        series_point: request.series_point,
    };

    let payload = match multipart.next_field().await? {
        Some(field) => {
            let name = field.name().unwrap_or_default();
            if name != "raw_data" {
                return Err(anyhow!(
                    "Second field in multipart upload must be `raw_data`, got `{}`",
                    name
                ))?;
            }
            Some(field)
        }
        None => None,
    };

    state
        .artifact_storage
        .write_artifact_version(version, payload)
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
