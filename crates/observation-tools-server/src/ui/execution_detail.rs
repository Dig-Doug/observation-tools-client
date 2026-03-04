//! Execution detail page handlers

use crate::api::observations::GetObservation;
use crate::api::AppError;
use crate::csrf::CsrfToken;
use crate::storage::MetadataStorage;
use crate::storage::StorageError;
use crate::ui::group_tree::{build_group_tree_view, PayloadTreeQuery};
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::ObservationId;
use std::sync::Arc;

/// Query parameters for execution detail page (log view)
#[derive(Debug, serde::Deserialize)]
pub struct ExecutionDetailQuery {
  /// Cursor-based page token
  pub page_token: Option<String>,

  /// Optional observation ID to display in side panel
  obs: Option<String>,
}

/// View type for execution detail page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionView {
  Log,
  Payload,
}

/// Execution detail page - Log view (shows all observations)
#[tracing::instrument(skip(metadata, templates))]
pub async fn execution_detail_log(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path(id): Path<String>,
  Query(query): Query<ExecutionDetailQuery>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!(execution_id = %id, "Rendering execution detail log page");
  let execution_id = ExecutionId::parse(&id)?;
  let execution = match metadata.get_execution(execution_id).await {
    Ok(execution) => Some(execution),
    Err(StorageError::NotFound(_)) => None,
    Err(e) => return Err(e.into()),
  };

  let page = metadata
    .get_observations(execution_id, query.page_token.clone(), None)
    .await?;
  let next_page_token = page.pagination.next_page_token;
  let previous_page_token = page.pagination.previous_page_token;
  let has_next_page = next_page_token.is_some();

  let observations: Vec<_> = page
    .observations
    .into_iter()
    .map(|obs| GetObservation::new(obs.observation, obs.payloads))
    .collect();

  let selected_observation = if let Some(obs_id) = &query.obs {
    let observation_id = ObservationId::parse(obs_id)?;
    match metadata.get_observation(observation_id).await {
      Ok(obs) => {
        let payloads = metadata.get_all_payloads(observation_id).await?;
        Some(GetObservation::new(obs, payloads))
      }
      Err(StorageError::NotFound(_)) => None,
      Err(e) => return Err(e.into()),
    }
  } else {
    None
  };

  let env = templates.acquire_env()?;
  let tmpl = env.get_template("execution_detail.html")?;
  let base_path = format!("/exe/{}", id);

  let html = tmpl.render(context! {
      execution => execution,
      execution_id => id,
      observations => observations,
      has_next_page => has_next_page,
      next_page_token => next_page_token,
      previous_page_token => previous_page_token,
      selected_observation => selected_observation,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
      view => "log",
      base_path => base_path,
  })?;

  Ok(Html(html))
}

/// Execution detail page - Payload view (shows group tree)
#[tracing::instrument(skip(metadata, templates))]
pub async fn execution_detail_payload(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path(id): Path<String>,
  Query(query): Query<PayloadTreeQuery>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!(execution_id = %id, "Rendering execution detail payload page");
  let execution_id = ExecutionId::parse(&id)?;
  let execution = match metadata.get_execution(execution_id).await {
    Ok(execution) => Some(execution),
    Err(StorageError::NotFound(_)) => None,
    Err(e) => return Err(e.into()),
  };

  // Build the group tree view
  let tree = build_group_tree_view(&metadata, execution_id, &query).await?;

  // If observation ID is provided, load the observation for the side panel
  let selected_observation = if let Some(obs_id) = &query.obs {
    let observation_id = ObservationId::parse(obs_id)?;
    match metadata.get_observation(observation_id).await {
      Ok(obs) => {
        let payloads = metadata.get_all_payloads(observation_id).await?;
        Some(GetObservation::new(obs, payloads))
      }
      Err(StorageError::NotFound(_)) => None,
      Err(e) => return Err(e.into()),
    }
  } else {
    None
  };

  let env = templates.acquire_env()?;
  let tmpl = env.get_template("execution_detail.html")?;
  let base_path = format!("/exe/{}/payload", id);

  let html = tmpl.render(context! {
      execution => execution,
      execution_id => id,
      tree => tree,
      selected_observation => selected_observation,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
      view => "payload",
      base_path => base_path,
  })?;

  Ok(Html(html))
}
