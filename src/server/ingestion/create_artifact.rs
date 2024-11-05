use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::permission::PermissionDataLoader;
use crate::auth::permission::PermissionLoader;
use crate::auth::principal::Principal;
use crate::ServerState;
use anyhow::anyhow;
use axum::async_trait;
use axum::extract::multipart::Field;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::extract::Multipart;
use axum::extract::State;
use axum::http::request::Parts;
use axum::routing::post;
use axum::Router;
use futures_util::stream::StreamExt;
use observation_tools_common::proto::CreateArtifactRequest;
use prost::Message;
use std::convert::Infallible;

#[derive(Clone)]
pub struct CreateArtifactState {
    pub permission_loader: PermissionDataLoader,
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

async fn create_artifact(
    State(state): State<CreateArtifactState>,
    principal: Principal,
    mut multipart: Multipart,
) -> Result<(), anyhow::Error> {
    let request = read_and_validate_request(multipart.next_field().await?).await?;

    let project_id = request.project_id.ok_or(anyhow!("Missing project_id"))?;
    let allowed = state
        .permission_loader
        .load_one(Permission::new(principal, project_id, Operation::Write))
        .await
        .map_err(|e| anyhow!("Error loading permission: {}", e))?;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }

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
    let unvalidated_request = CreateArtifactRequest::decode(data)?;
    Ok(unvalidated_request.validate()?)
}
