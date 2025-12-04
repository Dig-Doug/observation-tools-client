//! Observation detail page handler

use crate::api::AppError;
use crate::csrf::CsrfToken;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Html;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::models::ExecutionId;
use std::sync::Arc;

/// Observation detail (for the side panel)
#[tracing::instrument(skip(metadata, templates, headers))]
pub async fn observation_detail(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path((execution_id, observation_id)): Path<(String, String)>,
  headers: HeaderMap,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!(
      execution_id = %execution_id,
      observation_id = %observation_id,
      "Rendering observation detail page"
  );

  let _execution_id = ExecutionId::parse(&execution_id)?;
  let observation_id = observation_tools_shared::ObservationId::parse(&observation_id)?;

  let observations = metadata.get_observations(&[observation_id]).await?;

  let observation = observations.into_iter().next().ok_or_else(|| {
    crate::storage::StorageError::NotFound(format!("Observation {} not found", observation_id))
  })?;

  tracing::debug!(observation_name = %observation.name, "Retrieved observation for UI");

  let env = templates.acquire_env()?;

  // Check if this is an HTMX request (for side panel)
  let is_htmx_request = headers
    .get("hx-request")
    .and_then(|v| v.to_str().ok())
    .map(|v| v == "true")
    .unwrap_or(false);

  // Use partial template for HTMX requests, full template otherwise
  let template_name = if is_htmx_request {
    "observation_detail_partial.html"
  } else {
    "observation_detail.html"
  };

  let tmpl = env.get_template(template_name)?;

  let html = tmpl.render(context! {
      observation => observation,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
  })?;

  Ok(Html(html))
}
