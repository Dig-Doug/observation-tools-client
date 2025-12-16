//! Observation detail page handler

use crate::api::observations::GetObservation;
use crate::api::AppError;
use crate::csrf::CsrfToken;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use std::sync::Arc;

/// Observation detail (for the side panel)
#[tracing::instrument(skip(metadata, templates))]
pub async fn observation_detail(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path((execution_id, observation_id)): Path<(String, String)>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!(
      execution_id = %execution_id,
      observation_id = %observation_id,
      "Rendering observation detail page"
  );
  let parsed_observation_id = observation_tools_shared::ObservationId::parse(&observation_id)?;
  let observation = match metadata.get_observations(&[parsed_observation_id]).await {
    Ok(observations) => observations
      .into_iter()
      .next()
      .map(|obs| GetObservation::new(obs)),
    Err(crate::storage::StorageError::NotFound(_)) => {
      // The user may go to the observation page before it's uploaded. Since the page
      // auto-refreshes, we do not throw an error so it will show up once it's
      // available.
      None
    }
    Err(e) => return Err(e.into()),
  };
  let env = templates.acquire_env()?;
  let tmpl = env.get_template("observation_detail.html")?;
  let html = tmpl.render(context! {
      observation => observation,
      execution_id => execution_id,
      observation_id => observation_id,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
  })?;
  Ok(Html(html))
}
