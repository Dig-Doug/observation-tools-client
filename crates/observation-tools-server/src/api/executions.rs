//! Execution API handlers

use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::api::CreateExecutionRequest;
use observation_tools_shared::api::CreateExecutionResponse;
use observation_tools_shared::api::GetExecutionResponse;
use observation_tools_shared::api::ListExecutionsQuery;
use observation_tools_shared::api::ListExecutionsResponse;
use observation_tools_shared::models::ExecutionId;
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

/// List all executions
#[utoipa::path(
    get,
    path = "/api/exe",
    params(ListExecutionsQuery),
    responses(
        (status = 200, description = "List of executions", body = ListExecutionsResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "executions"
)]
#[tracing::instrument(skip(metadata))]
pub async fn list_executions(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Query(query): Query<ListExecutionsQuery>,
) -> Result<Json<ListExecutionsResponse>, AppError> {
  let limit = query.limit.unwrap_or(100);
  tracing::debug!(limit = limit, offset = ?query.offset, "Listing executions");

  // Fetch one extra to determine if there are more pages
  let mut executions = metadata
    .list_executions(Some(limit + 1), query.offset)
    .await?;

  let has_next_page = executions.len() > limit;
  if has_next_page {
    executions.pop(); // Remove the extra record
  }

  tracing::debug!(
    count = executions.len(),
    has_next_page = has_next_page,
    "Executions listed"
  );

  Ok(Json(ListExecutionsResponse {
    executions,
    has_next_page,
  }))
}

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
