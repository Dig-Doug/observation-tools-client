use crate::api::AppState;
use crate::api::{self};
use crate::config::Config;
use crate::csrf;
use crate::storage::LocalBlobStorage;
use crate::storage::SledStorage;
use crate::ui;
use axum::middleware;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use utoipa_swagger_ui::SwaggerUi;

/// The Observation Tools server
pub struct Server {
  config: Config,
}

impl Server {
  pub fn new(config: Config) -> Self {
    Self { config }
  }

  pub async fn run(self, listener: tokio::net::TcpListener) -> anyhow::Result<()> {
    info!("Starting Observation Tools server");
    debug!(data_dir = ?self.config.data_dir, "Initializing storage");
    let state = AppState {
      metadata: Arc::new(SledStorage::new(&self.config.data_dir.join("metadata"))?),
      blobs: Arc::new(LocalBlobStorage::new(&self.config.blob_dir)?),
      templates: ui::init_templates(),
    };

    let ui_router = Router::new()
      .route("/", get(ui::index))
      .route("/exe", get(ui::list_executions))
      .route("/exe/{id}", get(ui::execution_detail))
      .route(
        "/exe/{execution_id}/obs/{observation_id}",
        get(ui::observation_detail),
      )
      .layer(middleware::from_fn(csrf::ui_csrf_middleware))
      .nest_service("/static", {
        let static_dir = std::env::current_dir()?.join("crates/observation-tools-server/static");
        debug!(static_dir = ?static_dir, "Serving static files from directory");
        ServeDir::new(static_dir)
      });

    let api_secret = self.config.api_secret.clone();
    let (mutating_router, readonly_router, openapi) = api::build_api();
    let api_router = Router::new()
      .merge(
        mutating_router.layer(axum::middleware::from_fn(move |req, next| {
          crate::auth::api_key_middleware(api_secret.clone(), req, next)
        })),
      )
      .merge(readonly_router)
      .layer(middleware::from_fn(csrf::validate_csrf));
    let app = Router::new()
      .merge(ui_router)
      .merge(api_router)
      .merge(SwaggerUi::new("/api/swagger-ui").url("/api/openapi.json", openapi))
      .layer(TraceLayer::new_for_http())
      .with_state(state);

    let bound_addr = listener.local_addr()?;
    info!("Server listening on http://{}", bound_addr);

    axum::serve(listener, app).await?;

    Ok(())
  }
}
