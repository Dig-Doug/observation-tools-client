//! API handlers

pub mod executions;
pub mod observations;
pub mod types;

use crate::storage::BlobStorage;
use crate::storage::MetadataStorage;
use axum::extract::DefaultBodyLimit;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::models::*;
use std::sync::Arc;
use tracing::error;
use tracing::warn;
use types::*;
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
  Template(minijinja::Error),
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

impl From<minijinja::Error> for AppError {
  fn from(err: minijinja::Error) -> Self {
    AppError::Template(err)
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
        warn!(error = %err, "Shared error (bad request)");
        (StatusCode::BAD_REQUEST, err.to_string())
      }
      AppError::BadRequest(msg) => {
        warn!(error = %msg, "Bad request");
        (StatusCode::BAD_REQUEST, msg.clone())
      }
      AppError::Template(err) => {
        let mut full_error_text = format!("Template rendering error: {:#}", err);
        let mut e = &err as &dyn std::error::Error;
        while let Some(next_err) = e.source() {
          full_error_text += &format!("\ncaused by: {:#}", next_err);
          e = next_err;
        }
        error!(error = %err, "Template rendering error: {}", full_error_text);
        (StatusCode::INTERNAL_SERVER_ERROR, full_error_text)
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
    modifiers(&SecurityAddon),
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

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .description(Some("API key authentication. Set OBSERVATION_TOOLS_API_SECRET on the server to enable."))
                        .build(),
                ),
            );
        }
    }
}

/// Build the API router with separate read-only and mutating routes
pub fn build_router(state: AppState) -> Router {
  use observation_tools_shared::MAX_BLOB_SIZE;
  use observation_tools_shared::MAX_OBSERVATION_BATCH_SIZE;

  // Blob upload endpoint with calculated body size limit
  // Based on MAX_BLOB_SIZE (500MB) for very large payloads
  let blob_upload_route = Router::new()
    .route(
      "/exe/{execution_id}/obs/{observation_id}/blob",
      post(observations::upload_observation_blob),
    )
    .layer(DefaultBodyLimit::max(MAX_BLOB_SIZE));

  // Observation creation endpoint with calculated body size limit
  // Based on BATCH_SIZE * MAX_OBSERVATION_SIZE
  // = 100 observations * (64KB payload + 4KB metadata) ≈ 6.8MB
  let observation_create_route = Router::new()
    .route(
      "/exe/{execution_id}/obs",
      post(observations::create_observations),
    )
    .layer(DefaultBodyLimit::max(MAX_OBSERVATION_BATCH_SIZE));

  // Mutating routes (POST) - will have auth middleware applied
  let mutating_routes = Router::new()
    .route("/exe", post(executions::create_execution))
    .merge(observation_create_route)
    .merge(blob_upload_route)
    .with_state(state.clone());

  // Read-only routes (GET) - no auth required
  let read_only_routes = Router::new()
    .route("/exe", get(executions::list_executions))
    .route("/exe/{id}", get(executions::get_execution))
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
    .with_state(state);

  Router::new()
    .merge(mutating_routes)
    .merge(read_only_routes)
}

/// Build router for mutating routes only (used to apply auth middleware)
pub fn build_mutating_router(state: AppState) -> Router {
  use observation_tools_shared::MAX_BLOB_SIZE;
  use observation_tools_shared::MAX_OBSERVATION_BATCH_SIZE;

  let blob_upload_route = Router::new()
    .route(
      "/exe/{execution_id}/obs/{observation_id}/blob",
      post(observations::upload_observation_blob),
    )
    .layer(DefaultBodyLimit::max(MAX_BLOB_SIZE));

  let observation_create_route = Router::new()
    .route(
      "/exe/{execution_id}/obs",
      post(observations::create_observations),
    )
    .layer(DefaultBodyLimit::max(MAX_OBSERVATION_BATCH_SIZE));

  Router::new()
    .route("/exe", post(executions::create_execution))
    .merge(observation_create_route)
    .merge(blob_upload_route)
    .with_state(state)
}

/// Build router for read-only routes (no auth required)
pub fn build_readonly_router(state: AppState) -> Router {
  Router::new()
    .route("/exe", get(executions::list_executions))
    .route("/exe/{id}", get(executions::get_execution))
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
