//! Get observation blob handler

use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use crate::storage::PayloadData;
use crate::storage::StorageError;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::PayloadId;
use std::sync::Arc;

/// Get observation payload content
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs/{observation_id}/payload/{payload_id}/content",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ("observation_id" = String, Path, description = "Observation ID"),
        ("payload_id" = String, Path, description = "Payload ID")
    ),
    responses(
        (status = 200, description = "Payload content", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 404, description = "Payload not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata, blobs))]
pub async fn get_observation_blob(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path((execution_id, observation_id, payload_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
  let execution_id = ExecutionId::parse(&execution_id)?;
  let observation_id = ObservationId::parse(&observation_id)?;
  let payload_id = PayloadId::from(payload_id);
  let payload = metadata.get_payload(execution_id, observation_id, payload_id.clone()).await?;

  let content_type = HeaderValue::from_str(&payload.mime_type)
    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

  // If the payload data is inline (from prefix scan), return it directly
  if let PayloadData::Inline(ref data) = payload.data {
    return Ok((
      StatusCode::OK,
      [(header::CONTENT_TYPE, content_type)],
      data.clone(),
    ));
  }

  // Otherwise fetch from blob storage
  let blob = blobs.get_blob(observation_id, payload_id.clone()).await?;
  Ok((
    StatusCode::OK,
    [(header::CONTENT_TYPE, content_type)],
    blob.to_vec(),
  ))
}

/// Get observation blob content (legacy route for backward compat)
/// This is kept to support old URLs like /api/exe/{exec_id}/obs/{obs_id}/content
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
pub async fn get_observation_blob_legacy(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path((execution_id, observation_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
  let execution_id = ExecutionId::parse(&execution_id)?;
  let observation_id = ObservationId::parse(&observation_id)?;
  let payloads = metadata.get_all_payloads(execution_id, observation_id).await?;

  // Use the first payload
  let payload_meta = payloads.first().ok_or_else(|| {
    StorageError::NotFound(format!(
      "No payloads found for observation {}",
      observation_id
    ))
  })?;
  let payload = metadata
    .get_payload(execution_id, observation_id, payload_meta.id.clone())
    .await?;

  let content_type = HeaderValue::from_str(&payload.mime_type)
    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

  if let PayloadData::Inline(ref data) = payload.data {
    return Ok((
      StatusCode::OK,
      [(header::CONTENT_TYPE, content_type)],
      data.clone(),
    ));
  }

  let blob = blobs.get_blob(observation_id, payload.id.clone()).await?;
  Ok((
    StatusCode::OK,
    [(header::CONTENT_TYPE, content_type)],
    blob.to_vec(),
  ))
}
