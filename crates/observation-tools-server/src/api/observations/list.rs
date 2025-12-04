//! List observations handler

use crate::api::types::ListObservationsQuery;
use crate::api::types::ListObservationsResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use std::sync::Arc;

/// List observations for an execution
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ListObservationsQuery
    ),
    responses(
        (status = 200, description = "List of observations", body = ListObservationsResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata))]
pub async fn list_observations(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path(execution_id): Path<String>,
  Query(query): Query<ListObservationsQuery>,
) -> Result<Json<ListObservationsResponse>, AppError> {
  let execution_id = ExecutionId::parse(&execution_id)?;
  let limit = query.limit.unwrap_or(100);

  tracing::debug!(
      execution_id = %execution_id,
      limit = limit,
      offset = ?query.offset,
      "Listing observations"
  );

  // Fetch one extra to determine if there are more pages
  let mut observations = metadata
    .list_observations(execution_id, Some(limit + 1), query.offset, None)
    .await?;

  let has_next_page = observations.len() > limit;
  if has_next_page {
    observations.pop(); // Remove the extra record
  }

  tracing::debug!(
    count = observations.len(),
    has_next_page = has_next_page,
    "Observations listed"
  );

  Ok(Json(ListObservationsResponse {
    observations,
    has_next_page,
  }))
}
