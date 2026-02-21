//! Create observations handler

use crate::api::types::CreateObservationsResponse;
use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use crate::storage::ObservationWithPayloads;
use crate::storage::PayloadData;
use crate::storage::StoredPayload;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::Observation;
use observation_tools_shared::PayloadId;
use observation_tools_shared::BLOB_THRESHOLD_BYTES;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Entry in the payload manifest sent by new clients
#[derive(Debug, Deserialize)]
struct PayloadManifestEntry {
  observation_id: String,
  payload_id: String,
  #[allow(dead_code)]
  name: String,
  mime_type: String,
  #[allow(dead_code)]
  size: usize,
}

/// Create observations (batch) via multipart form
///
/// The multipart form should contain:
/// - "observations": JSON array of observation metadata
/// - "{obs_id}:{payload_id}:{name}": Binary payload data for each payload
/// - Legacy: "{obs_id}:{name}" or "{obs_id}" formats are also supported
#[tracing::instrument(skip(metadata, blobs, multipart))]
pub async fn create_observations(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path(execution_id): Path<String>,
  mut multipart: Multipart,
) -> Result<Json<CreateObservationsResponse>, AppError> {
  let _execution_id = ExecutionId::parse(&execution_id)?;

  let mut observations: Option<Vec<Observation>> = None;
  let mut payload_manifest: Option<Vec<PayloadManifestEntry>> = None;
  let mut payloads: HashMap<String, bytes::Bytes> = HashMap::new();

  // Parse all multipart fields
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
  {
    let name = field.name().unwrap_or_default().to_string();

    if name == "observations" {
      // Parse JSON observations metadata
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read observations data: {}", e)))?;
      let parsed: Vec<Observation> = serde_json::from_slice(&data)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse observations JSON: {}", e)))?;
      observations = Some(parsed);
    } else if name == "payload_manifest" {
      // Parse payload manifest (new clients send this for authoritative MIME types)
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read payload manifest: {}", e)))?;
      let parsed: Vec<PayloadManifestEntry> = serde_json::from_slice(&data)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse payload manifest JSON: {}", e)))?;
      payload_manifest = Some(parsed);
    } else {
      // This is a payload field
      let data = field.bytes().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read payload data for {}: {}", name, e))
      })?;
      payloads.insert(name, data);
    }
  }

  let observations = observations.ok_or_else(|| {
    AppError::BadRequest("Missing 'observations' field in multipart form".to_string())
  })?;

  tracing::debug!(
    execution_id = %execution_id,
    count = observations.len(),
    payload_count = payloads.len(),
    "Creating observations batch"
  );

  // Build a lookup from (observation_id, payload_id) -> manifest entry for MIME types
  let manifest_lookup: HashMap<(String, String), &PayloadManifestEntry> = payload_manifest
    .as_ref()
    .map(|entries| {
      entries
        .iter()
        .map(|e| ((e.observation_id.clone(), e.payload_id.clone()), e))
        .collect()
    })
    .unwrap_or_default();

  // Build ObservationWithPayloads for each observation by collecting all matching payloads
  let mut observations_with_payloads = Vec::with_capacity(observations.len());

  for obs in &observations {
    let obs_id_str = obs.id.to_string();
    let mut obs_payloads: Vec<StoredPayload> = Vec::new();

    // Collect all payload keys that belong to this observation
    let matching_keys: Vec<String> = payloads
      .keys()
      .filter(|k| {
        let id_part = k.split(':').next().unwrap_or(k);
        id_part == obs_id_str
      })
      .cloned()
      .collect();

    for key in matching_keys {
      let data = payloads.remove(&key).expect("key was just found");
      let (payload_id, name) = parse_payload_key(&key, &obs_id_str)?;

      // Determine MIME type: check manifest first, fall back to heuristic for old clients
      let mime_type = if let Some(entry) = manifest_lookup.get(&(obs_id_str.clone(), payload_id.as_str().to_string())) {
        entry.mime_type.clone()
      } else if serde_json::from_slice::<serde_json::Value>(&data).is_ok() {
        "application/json".to_string()
      } else if std::str::from_utf8(&data).is_ok() {
        "text/plain".to_string()
      } else {
        "application/octet-stream".to_string()
      };

      let size = data.len();
      let payload_data = if size >= BLOB_THRESHOLD_BYTES {
        blobs
          .store_blob(obs.id, payload_id.clone(), data.clone())
          .await?;
        PayloadData::Blob
      } else {
        PayloadData::Inline(data.to_vec())
      };

      obs_payloads.push(StoredPayload {
        id: payload_id,
        name,
        mime_type,
        size,
        data: payload_data,
      });
    }

    if obs_payloads.is_empty() {
      return Err(AppError::BadRequest(format!(
        "Missing payload data for observation ID {}",
        obs.id
      )));
    }

    observations_with_payloads.push(ObservationWithPayloads {
      observation: obs.clone(),
      payloads: obs_payloads,
    });
  }

  // Warn about any orphaned payloads
  for key in payloads.keys() {
    tracing::warn!(
      payload_key = %key,
      "Received payload for unknown observation ID"
    );
  }

  metadata
    .store_observations(observations_with_payloads)
    .await?;

  tracing::info!(
    execution_id = %execution_id,
    count = observations.len(),
    "Observations created successfully"
  );

  Ok(Json(CreateObservationsResponse {}))
}

/// Parse a payload key into (PayloadId, name).
/// Supports formats:
/// - "{obs_id}:{payload_id}:{name}" (new format)
/// - "{obs_id}:{name}" (legacy named format, generates new PayloadId)
/// - "{obs_id}" (legacy bare format, generates new PayloadId, name = "default")
fn parse_payload_key(
  key: &str,
  obs_id_str: &str,
) -> Result<(PayloadId, String), AppError> {
  let rest = &key[obs_id_str.len()..];
  if rest.is_empty() {
    // Legacy bare format: "{obs_id}"
    return Ok((PayloadId::new(), "default".to_string()));
  }

  if !rest.starts_with(':') {
    return Err(AppError::BadRequest(format!(
      "Invalid payload key format: {}",
      key
    )));
  }

  let after_obs_id = &rest[1..]; // skip the ':'
  let parts: Vec<&str> = after_obs_id.splitn(2, ':').collect();

  match parts.len() {
    1 => {
      // Legacy "{obs_id}:{name}" format
      Ok((PayloadId::new(), parts[0].to_string()))
    }
    2 => {
      // "{obs_id}:{payload_id}:{name}" - try parsing first part as PayloadId
      if uuid::Uuid::parse_str(parts[0]).is_ok() {
        let payload_id = PayloadId::from(parts[0]);
        Ok((payload_id, parts[1].to_string()))
      } else {
        // Not a valid PayloadId, treat as legacy format where the whole thing is a name
        // This shouldn't normally happen
        Ok((PayloadId::new(), after_obs_id.to_string()))
      }
    }
    _ => Err(AppError::BadRequest(format!(
      "Invalid payload key format: {}",
      key
    ))),
  }
}
