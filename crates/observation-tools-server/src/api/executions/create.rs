//! Create execution handler

use crate::api::types::CreateExecutionRequest;
use crate::api::types::CreateExecutionResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

/// Create a new execution
#[utoipa::path(
    post,
    path = "/api/exe",
    request_body = CreateExecutionRequest,
    responses(
        (status = 200, description = "Execution created successfully", body = CreateExecutionResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "executions"
)]
#[tracing::instrument(skip(metadata), fields(execution_id))]
pub async fn create_execution(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Json(req): Json<CreateExecutionRequest>,
) -> Result<Json<CreateExecutionResponse>, AppError> {
  let execution = req.execution;
  tracing::debug!(name = %execution.name, "Creating new execution");

  tracing::Span::current().record("execution_id", tracing::field::display(&execution.id));

  metadata.store_execution(&execution).await?;

  tracing::info!(execution_id = %execution.id, "Execution created successfully");

  Ok(Json(CreateExecutionResponse {}))
}
