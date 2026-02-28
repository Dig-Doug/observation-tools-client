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
  /// Maximum number of results to return
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<usize>,

  /// Number of results to skip (for pagination)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset: Option<usize>,

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

  let limit = query.limit.unwrap_or(100);
  let offset = query.offset.unwrap_or(0);

  let total_count = metadata
    .count_observations(execution_id, None)
    .await?;

  let mut observations = metadata
    .list_observations(execution_id, Some(limit + 1), Some(offset), None)
    .await?;
  let has_next_page = observations.len() > limit;
  if has_next_page {
    observations.pop();
  }
  let page = (offset / limit) + 1;

  let observations: Vec<_> = observations
    .into_iter()
    .map(|obs| GetObservation::new(obs))
    .collect();

  let selected_observation = if let Some(obs_id) = &query.obs {
    let observation_id = ObservationId::parse(obs_id)?;
    match metadata.get_observation(observation_id).await {
      Ok(obs) => Some(GetObservation::new(obs)),
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
      total_count => total_count,
      offset => offset,
      limit => limit,
      page => page,
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
      Ok(obs) => Some(GetObservation::new(obs)),
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
