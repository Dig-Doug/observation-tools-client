//! HTTP server implementation

use crate::api::ApiDoc;
use crate::api::AppState;
use crate::api::{self};
use crate::auth::{self, AuthState, OAuthConfig, OAuthManager, Provider, SledAuthStorage};
use crate::config::Config;
use crate::csrf;
use crate::storage::LocalBlobStorage;
use crate::storage::SledStorage;
use crate::ui;
use axum::middleware;
use axum::routing::{get, post};
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

    // Initialize auth storage (shares same sled db as metadata)
    let auth_db = sled::open(&self.config.data_dir.join("metadata"))?;
    let auth_storage: Arc<dyn crate::auth::AuthStorage> = Arc::new(SledAuthStorage::new(auth_db));
    tracing::info!("Auth storage initialized");

    // Initialize OAuth manager
    let mut oauth_manager = OAuthManager::new();
    if let Some(google_config) = &self.config.google_oauth {
      let redirect_url = format!("{}/auth/google/callback", self.config.base_url);
      oauth_manager.add_provider(
        Provider::Google,
        OAuthConfig {
          client_id: google_config.client_id.clone(),
          client_secret: google_config.client_secret.clone(),
          redirect_url,
        },
      )?;
      tracing::info!("Google OAuth configured");
    }
    if let Some(github_config) = &self.config.github_oauth {
      let redirect_url = format!("{}/auth/github/callback", self.config.base_url);
      oauth_manager.add_provider(
        Provider::GitHub,
        OAuthConfig {
          client_id: github_config.client_id.clone(),
          client_secret: github_config.client_secret.clone(),
          redirect_url,
        },
      )?;
      tracing::info!("GitHub OAuth configured");
    }
    let oauth_manager = Arc::new(oauth_manager);

    // Initialize template environment
    let templates = ui::init_templates();
    tracing::debug!("Template environment initialized");

    let state = AppState {
      metadata: metadata.clone(),
      blobs,
      templates,
    };

    let auth_state = AuthState {
      auth_storage: auth_storage.clone(),
      oauth_manager: oauth_manager.clone(),
    };

    // Build auth router
    let auth_router = Router::new()
      .route("/auth/login", get(auth::login_page))
      .route("/auth/:provider", get(auth::oauth_login))
      .route("/auth/:provider/callback", get(auth::oauth_callback))
      .route(
        "/auth/logout",
        post(auth::logout).layer(middleware::from_fn(auth::require_auth)),
      )
      .with_state(auth_state);

    // Build UI router with CSRF middleware
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

    // Build the main router with user extraction middleware
    let app = Router::new()
      .merge(ui_router)
      .merge(auth_router)
      .nest(
        "/api",
        api::build_router(state).layer(middleware::from_fn(csrf::validate_csrf)),
      )
      .merge(SwaggerUi::new("/api/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
      .nest_service("/static", serve_static)
      .layer(middleware::from_fn_with_state(
        auth_storage.clone(),
        auth::extract_user::<Arc<dyn crate::auth::AuthStorage>>,
      ))
      .layer(TraceLayer::new_for_http());

    tracing::debug!("Router configured");

    // Log the actual bound address
    let bound_addr = listener.local_addr()?;
    tracing::info!("Server listening on http://{}", bound_addr);

    axum::serve(listener, app).await?;

    Ok(())
  }
}
