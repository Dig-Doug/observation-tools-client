//! Observation API handlers

use crate::api::AppError;
use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use observation_tools_shared::api::CreateObservationsRequest;
use observation_tools_shared::api::CreateObservationsResponse;
use observation_tools_shared::api::GetObservationResponse;
use observation_tools_shared::api::ListObservationsQuery;
use observation_tools_shared::api::ListObservationsResponse;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::ObservationId;
use std::sync::Arc;
use tracing::trace;

/// Create observations (batch)
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

/// Get observation blob content
#[tracing::instrument(skip(blobs))]
pub async fn get_observation_blob(
    State(blobs): State<Arc<dyn BlobStorage>>,
    Path((_execution_id, observation_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    tracing::debug!(observation_id = %observation_id, "Getting observation blob");

    let observation_id = ObservationId::parse(&observation_id)?;

    let blob = blobs.get_blob(observation_id).await?;

    tracing::debug!(size = blob.len(), "Blob retrieved");

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        blob,
    ))
}
