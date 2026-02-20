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
use axum::Json;
use axum::Router;
use minijinja_autoreload::AutoReloader;
use std::sync::Arc;
use tracing::error;
use tracing::warn;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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

/// Build the complete API router and OpenAPI spec
/// Returns the router (split into mutating and readonly) and the OpenAPI spec
pub fn build_api() -> (Router<AppState>, Router<AppState>, OpenApi) {
  use observation_tools_shared::MAX_OBSERVATION_BATCH_SIZE;
  use utoipa::openapi::security::HttpAuthScheme;
  use utoipa::openapi::security::HttpBuilder;
  use utoipa::openapi::security::SecurityScheme;

  let (mutation_router, mutation_openapi) = OpenApiRouter::<AppState>::new()
    .routes(routes!(executions::create_execution))
    .split_for_parts();

  // create_observations uses multipart form which isn't supported by OpenAPI
  // codegen, so we register it manually outside the OpenApiRouter
  let create_observations_route = Router::new()
    .route(
      "/api/exe/{execution_id}/obs",
      axum::routing::post(observations::create_observations),
    )
    .layer(DefaultBodyLimit::max(MAX_OBSERVATION_BATCH_SIZE));

  let mutation_router = Router::new()
    .merge(mutation_router.layer(DefaultBodyLimit::max(MAX_OBSERVATION_BATCH_SIZE)))
    .merge(create_observations_route);

  let (read_only_router, read_only_openapi) = OpenApiRouter::<AppState>::new()
    .routes(routes!(executions::list_executions))
    .routes(routes!(executions::get_execution))
    .routes(routes!(observations::list_observations))
    .routes(routes!(observations::get_observation))
    .routes(routes!(observations::get_observation_blob))
    .routes(routes!(observations::get_observation_blob_legacy))
    .split_for_parts();

  let mut openapi = OpenApi::default();
  openapi.merge(mutation_openapi);
  openapi.merge(read_only_openapi);

  // Add security scheme
  let components = openapi.components.get_or_insert_with(Default::default);
  components.add_security_scheme(
    "bearer_auth",
    SecurityScheme::Http(
      HttpBuilder::new()
        .scheme(HttpAuthScheme::Bearer)
        .bearer_format("API Key")
        .description(Some(
          "API key authentication. Set OBSERVATION_TOOLS_API_SECRET on the server to enable.",
        ))
        .build(),
    ),
  );

  (mutation_router, read_only_router, openapi)
}
