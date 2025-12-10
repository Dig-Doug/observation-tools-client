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

  let _parsed_execution_id = ExecutionId::parse(&execution_id)?;
  let parsed_observation_id = observation_tools_shared::ObservationId::parse(&observation_id)?;

  // Try to get the observation, but handle not found gracefully
  let observation = match metadata.get_observations(&[parsed_observation_id]).await {
    Ok(observations) => observations.into_iter().next(),
    Err(crate::storage::StorageError::NotFound(_)) => None,
    Err(e) => return Err(e.into()),
  };

  if let Some(ref obs) = observation {
    tracing::debug!(observation_name = %obs.name, "Retrieved observation for UI");
  } else {
    tracing::debug!(
      observation_id = %observation_id,
      "Observation not found, rendering waiting page"
    );
  }

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
      execution_id => execution_id,
      observation_id => observation_id,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
  })?;

  Ok(Html(html))
}
