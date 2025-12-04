//! Upload observation blob handler

use crate::api::AppError;
use crate::storage::BlobStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use observation_tools_shared::models::ObservationId;
use std::sync::Arc;

/// Upload observation blob content
///
/// Note: This endpoint is not included in the OpenAPI spec because progenitor
/// doesn't support binary request bodies. The client implements this manually.
#[tracing::instrument(skip(blobs, data))]
pub async fn upload_observation_blob(
  State(blobs): State<Arc<dyn BlobStorage>>,
  Path((_execution_id, observation_id)): Path<(String, String)>,
  data: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
  tracing::debug!(
    observation_id = %observation_id,
    size = data.len(),
    "Uploading observation blob"
  );

  let observation_id = ObservationId::parse(&observation_id)?;

  // Store the blob
  blobs.store_blob(observation_id, data).await?;

  tracing::info!(
    observation_id = %observation_id,
    "Blob uploaded successfully"
  );

  Ok(StatusCode::OK)
}
