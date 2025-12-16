//! Get observation blob handler

use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use crate::storage::StorageError;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use observation_tools_shared::ObservationId;
use std::sync::Arc;

/// Get observation blob content
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs/{observation_id}/content",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ("observation_id" = String, Path, description = "Observation ID")
    ),
    responses(
        (status = 200, description = "Observation blob content", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 404, description = "Observation blob not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata, blobs))]
pub async fn get_observation_blob(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path((_execution_id, observation_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
  let observation_id = ObservationId::parse(&observation_id)?;
  let observations = metadata.get_observations(&[observation_id]).await?;
  let observation = observations
    .into_iter()
    .next()
    .ok_or_else(|| StorageError::NotFound(format!("Observation {} not found", observation_id)))?;
  let content_type = HeaderValue::from_str(&observation.observation.mime_type)
    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
  if let Some(payload) = observation.payload_or_pointer.payload {
    return Ok((
      StatusCode::OK,
      [(header::CONTENT_TYPE, content_type)],
      payload,
    ));
  }
  let blob = blobs.get_blob(observation_id).await?;
  Ok((
    StatusCode::OK,
    [(header::CONTENT_TYPE, content_type)],
    blob.to_vec(),
  ))
}
