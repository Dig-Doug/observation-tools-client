//! Create observations handler

use crate::api::types::CreateObservationsRequest;
use crate::api::types::CreateObservationsResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use std::sync::Arc;

/// Create observations (batch)
#[utoipa::path(
    post,
    path = "/api/exe/{execution_id}/obs",
    params(
        ("execution_id" = String, Path, description = "Execution ID")
    ),
    request_body = CreateObservationsRequest,
    responses(
        (status = 200, description = "Observations created successfully", body = CreateObservationsResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata, req), fields(observation_count = req.observations.len()))]
pub async fn create_observations(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path(execution_id): Path<String>,
  Json(req): Json<CreateObservationsRequest>,
) -> Result<Json<CreateObservationsResponse>, AppError> {
  tracing::debug!(
      execution_id = %execution_id,
      count = req.observations.len(),
      "Creating observations batch"
  );

  let _execution_id = ExecutionId::parse(&execution_id)?;

  // Store observations (don't check if execution exists - uploads may be out of
  // order)
  metadata.store_observations(&req.observations).await?;

  tracing::info!(
      execution_id = %execution_id,
      count = req.observations.len(),
      "Observations created successfully"
  );

  Ok(Json(CreateObservationsResponse {}))
}
