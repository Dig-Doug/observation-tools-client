//! Get observation handler

use crate::api::types::GetObservationResponse;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::ObservationId;
use std::sync::Arc;

/// Get a single observation
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs/{observation_id}",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ("observation_id" = String, Path, description = "Observation ID")
    ),
    responses(
        (status = 200, description = "Observation details", body = GetObservationResponse),
        (status = 404, description = "Observation not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata))]
pub async fn get_observation(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path((execution_id, observation_id)): Path<(String, String)>,
) -> Result<Json<GetObservationResponse>, AppError> {
  tracing::debug!(
      execution_id = %execution_id,
      observation_id = %observation_id,
      "Getting observation"
  );

  let _execution_id = ExecutionId::parse(&execution_id)?;
  let observation_id = ObservationId::parse(&observation_id)?;

  let observations = metadata.get_observations(&[observation_id]).await?;

  let observation = observations.into_iter().next().ok_or_else(|| {
    crate::storage::StorageError::NotFound(format!("Observation {} not found", observation_id))
  })?;

  tracing::debug!(name = %observation.name, "Observation retrieved");

  Ok(Json(GetObservationResponse { observation }))
}
