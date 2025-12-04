//! Get execution handler

use crate::api::types::GetExecutionResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use std::sync::Arc;

/// Get a single execution
#[utoipa::path(
    get,
    path = "/api/exe/{id}",
    params(
        ("id" = String, Path, description = "Execution ID")
    ),
    responses(
        (status = 200, description = "Execution details", body = GetExecutionResponse),
        (status = 404, description = "Execution not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "executions"
)]
#[tracing::instrument(skip(metadata))]
pub async fn get_execution(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path(id): Path<String>,
) -> Result<Json<GetExecutionResponse>, AppError> {
  tracing::debug!(id = %id, "Getting execution");

  let execution_id = ExecutionId::parse(&id)?;
  let execution = metadata.get_execution(execution_id).await?;

  tracing::debug!(name = %execution.name, "Execution retrieved");

  Ok(Json(GetExecutionResponse { execution }))
}
