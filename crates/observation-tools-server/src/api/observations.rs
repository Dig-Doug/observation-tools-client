//! Observation API handlers

use crate::api::types::CreateObservationsRequest;
use crate::api::types::CreateObservationsResponse;
use crate::api::types::GetObservationResponse;
use crate::api::types::ListObservationsQuery;
use crate::api::types::ListObservationsResponse;
use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::header;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::ObservationId;
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
    .list_observations(execution_id, Some(limit + 1), query.offset)
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
