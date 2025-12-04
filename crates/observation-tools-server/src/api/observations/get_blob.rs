//! Get observation blob handler

use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use observation_tools_shared::models::ObservationId;
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
  tracing::debug!(observation_id = %observation_id, "Getting observation blob");

  let observation_id = ObservationId::parse(&observation_id)?;

  // First, retrieve the observation to check if payload is set
  let observations = metadata.get_observations(&[observation_id]).await?;
  let observation = observations.into_iter().next().ok_or_else(|| {
    crate::storage::StorageError::NotFound(format!("Observation {} not found", observation_id))
  })?;

  // Get the mime type from the observation
  let mime_type = observation.payload.mime_type.clone();
  let content_type = HeaderValue::from_str(&mime_type)
    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

  // If payload data is set (non-empty), return it directly
  if !observation.payload.data.is_empty() {
    tracing::debug!(
      size = observation.payload.size,
      mime_type = %mime_type,
      "Returning inline payload"
    );

    let data = observation.payload.data.into_bytes();
    return Ok((
      StatusCode::OK,
      [(header::CONTENT_TYPE, content_type)],
      data.into(),
    ));
  }

  // Otherwise, fetch from blob storage
  tracing::debug!(mime_type = %mime_type, "Fetching from blob storage");
  let blob = blobs.get_blob(observation_id).await?;

  tracing::debug!(size = blob.len(), "Blob retrieved from storage");

  Ok((StatusCode::OK, [(header::CONTENT_TYPE, content_type)], blob))
}
