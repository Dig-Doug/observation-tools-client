//! API handlers

pub mod executions;
pub mod observations;

use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::api::*;
use observation_tools_shared::models::*;
use std::sync::Arc;
use utoipa::OpenApi;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
  pub metadata: Arc<dyn MetadataStorage>,
  pub blobs: Arc<dyn BlobStorage>,
  pub templates: Arc<AutoReloader>,
}

impl FromRef<AppState> for Arc<dyn MetadataStorage> {
  fn from_ref(state: &AppState) -> Self {
    state.metadata.clone()
  }
}

impl FromRef<AppState> for Arc<dyn BlobStorage> {
  fn from_ref(state: &AppState) -> Self {
    state.blobs.clone()
  }
}

impl FromRef<AppState> for Arc<AutoReloader> {
  fn from_ref(state: &AppState) -> Self {
    state.templates.clone()
  }
}

/// Application error type
#[derive(Debug)]
pub enum AppError {
  Storage(crate::storage::StorageError),
  Shared(observation_tools_shared::Error),
  BadRequest(String),
}

impl From<crate::storage::StorageError> for AppError {
  fn from(err: crate::storage::StorageError) -> Self {
    AppError::Storage(err)
  }
}

impl From<observation_tools_shared::Error> for AppError {
  fn from(err: observation_tools_shared::Error) -> Self {
    AppError::Shared(err)
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, message) = match &self {
      AppError::Storage(crate::storage::StorageError::NotFound(msg)) => {
        tracing::debug!(error = %msg, "Resource not found");
        (StatusCode::NOT_FOUND, msg.clone())
      }
      AppError::Storage(err) => {
        tracing::error!(error = %err, "Storage error");
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
      }
      AppError::Shared(err) => {
        tracing::warn!(error = %err, "Shared error (bad request)");
        (StatusCode::BAD_REQUEST, err.to_string())
      }
      AppError::BadRequest(msg) => {
        tracing::warn!(error = %msg, "Bad request");
        (StatusCode::BAD_REQUEST, msg.clone())
      }
    };

    let body = serde_json::json!({
        "error": message
    });

    (status, Json(body)).into_response()
  }
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        executions::create_execution,
        executions::list_executions,
        executions::get_execution,
        observations::create_observations,
        observations::list_observations,
        observations::get_observation,
        observations::get_observation_blob,
    ),
    components(
        schemas(
            Execution,
            ExecutionId,
            Observation,
            ObservationId,
            SourceInfo,
            Payload,
            CreateExecutionRequest,
            CreateExecutionResponse,
            ListExecutionsQuery,
            ListExecutionsResponse,
            GetExecutionResponse,
            CreateObservationsRequest,
            CreateObservationsResponse,
            ListObservationsQuery,
            ListObservationsResponse,
            GetObservationResponse,
            ErrorResponse,
        )
    ),
    tags(
        (name = "executions", description = "Execution management endpoints"),
        (name = "observations", description = "Observation management endpoints")
    ),
    info(
        title = "Observation Tools API",
        version = "0.1.0",
        description = "API for the Observation Tools developer data inspection toolkit"
    )
)]
pub struct ApiDoc;

/// Build the API router
pub fn build_router(state: AppState) -> Router {
  Router::new()
    // Execution routes
    .route("/exe", post(executions::create_execution))
    .route("/exe", get(executions::list_executions))
    .route("/exe/{id}", get(executions::get_execution))
    // Observation routes
    .route(
      "/exe/{execution_id}/obs",
      post(observations::create_observations),
    )
    .route(
      "/exe/{execution_id}/obs",
      get(observations::list_observations),
    )
    .route(
      "/exe/{execution_id}/obs/{observation_id}",
      get(observations::get_observation),
    )
    .route(
      "/exe/{execution_id}/obs/{observation_id}/content",
      get(observations::get_observation_blob),
    )
    .with_state(state)
}
