//! HTTP server implementation

use crate::api::ApiDoc;
use crate::api::AppState;
use crate::api::{self};
use crate::config::Config;
use crate::storage::LocalBlobStorage;
use crate::storage::SledStorage;
use crate::ui;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// The Observation Tools server
pub struct Server {
  config: Config,
}

impl Server {
  /// Create a new server with the given configuration
  pub fn new(config: Config) -> Self {
    Self { config }
  }

  /// Run the server with the given TCP listener
  pub async fn run(self, listener: tokio::net::TcpListener) -> anyhow::Result<()> {
    tracing::info!("Starting Observation Tools server");
    tracing::debug!(data_dir = ?self.config.data_dir, "Initializing storage");

    // Initialize storage
    let metadata = Arc::new(SledStorage::new(&self.config.data_dir.join("metadata"))?);
    tracing::info!("Metadata storage initialized");

    let blobs = Arc::new(LocalBlobStorage::new(&self.config.blob_dir)?);
    tracing::info!(blob_dir = ?self.config.blob_dir, "Blob storage initialized");

    // Initialize template environment
    let templates = ui::init_templates();
    tracing::debug!("Template environment initialized");

    let state = AppState {
      metadata: metadata.clone(),
      blobs,
      templates,
    };

    // Build UI router
    let ui_router = Router::new()
      .route("/", get(ui::index))
      .route("/exe", get(ui::list_executions))
      .route("/exe/{id}", get(ui::execution_detail))
      .route(
        "/exe/{execution_id}/obs/{observation_id}",
        get(ui::observation_detail),
      )
      .layer(middleware::from_fn(csrf::ui_csrf_middleware))
      .with_state(state.clone());

    // Serve static files
    let static_dir = std::env::current_dir()?.join("crates/observation-tools-server/static");
    tracing::debug!(static_dir = ?static_dir, "Serving static files from directory");
    let serve_static = ServeDir::new(static_dir);

    // Build API router with optional authentication
    let api_router = if let Some(ref secret) = self.config.api_secret {
      let secret_clone = secret.clone();
      api::build_router(state)
        .layer(axum::middleware::from_fn(move |req, next| {
          crate::auth::api_key_middleware(secret_clone.clone(), req, next)
        }))
    } else {
      api::build_router(state)
    };

    // Build the main router
    let app = Router::new()
      .merge(ui_router)
      .nest(
        "/api",
        api::build_router(state).layer(middleware::from_fn(csrf::validate_csrf)),
      )
      .merge(SwaggerUi::new("/api/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
      .nest_service("/static", serve_static)
      .layer(TraceLayer::new_for_http());

    tracing::debug!("Router configured");

    // Log the actual bound address
    let bound_addr = listener.local_addr()?;
    tracing::info!("Server listening on http://{}", bound_addr);

    axum::serve(listener, app).await?;

    Ok(())
  }
}
