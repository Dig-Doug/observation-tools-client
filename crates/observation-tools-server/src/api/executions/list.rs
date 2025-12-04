//! List executions handler

use crate::api::types::ListExecutionsQuery;
use crate::api::types::ListExecutionsResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

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
