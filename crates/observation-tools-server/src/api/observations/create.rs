//! Create observations handler

use crate::api::types::CreateObservationsResponse;
use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::Observation;
use observation_tools_shared::BLOB_THRESHOLD_BYTES;
use std::collections::HashMap;
use std::sync::Arc;

/// Create observations (batch) via multipart form
///
/// The multipart form should contain:
/// - "observations": JSON array of observation metadata (with empty payload.data)
/// - "{observation_id}": Binary payload data for each observation
#[tracing::instrument(skip(metadata, blobs, multipart))]
pub async fn create_observations(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path(execution_id): Path<String>,
  mut multipart: Multipart,
) -> Result<Json<CreateObservationsResponse>, AppError> {
  let _execution_id = ExecutionId::parse(&execution_id)?;

  let mut observations: Option<Vec<Observation>> = None;
  let mut payloads: HashMap<String, bytes::Bytes> = HashMap::new();

  // Parse all multipart fields
  while let Some(field) = multipart.next_field().await.map_err(|e| {
    AppError::BadRequest(format!("Failed to read multipart field: {}", e))
  })? {
    let name = field.name().unwrap_or_default().to_string();

    if name == "observations" {
      // Parse JSON observations metadata
      let data = field.bytes().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read observations data: {}", e))
      })?;
      let parsed: Vec<Observation> = serde_json::from_slice(&data).map_err(|e| {
        AppError::BadRequest(format!("Failed to parse observations JSON: {}", e))
      })?;
      observations = Some(parsed);
    } else {
      // This is a payload field, keyed by observation ID
      let data = field.bytes().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read payload data for {}: {}", name, e))
      })?;
      payloads.insert(name, data);
    }
  }

  let mut observations = observations.ok_or_else(|| {
    AppError::BadRequest("Missing 'observations' field in multipart form".to_string())
  })?;

  tracing::debug!(
    execution_id = %execution_id,
    count = observations.len(),
    payload_count = payloads.len(),
    "Creating observations batch"
  );

  // Merge payload data into observations and handle blob storage
  for obs in &mut observations {
    let obs_id_str = obs.id.to_string();
    if let Some(payload_data) = payloads.remove(&obs_id_str) {
      let payload_size = payload_data.len();

      if payload_size >= BLOB_THRESHOLD_BYTES {
        // Store large payloads in blob storage
        tracing::debug!(
          observation_id = %obs.id,
          size = payload_size,
          "Storing large payload in blob storage"
        );
        blobs.store_blob(obs.id, payload_data).await?;
        // Keep payload.data empty, payload.size should already be set by client
      } else {
        // Store small payloads inline
        obs.payload.data = payload_data.to_vec();
        obs.payload.size = payload_size;
      }
    }
  }

  // Warn about any orphan payloads
  for orphan_id in payloads.keys() {
    tracing::warn!(
      observation_id = %orphan_id,
      "Received payload for unknown observation ID"
    );
  }

  // Store observations
  metadata.store_observations(&observations).await?;

  tracing::info!(
    execution_id = %execution_id,
    count = observations.len(),
    "Observations created successfully"
  );

  Ok(Json(CreateObservationsResponse {}))
}
