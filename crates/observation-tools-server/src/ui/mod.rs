//! Web UI handlers

use crate::api::types::ListExecutionsQuery;
use crate::api::types::ListObservationsQuery;
use crate::api::AppError;
use crate::csrf::CsrfToken;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Html;
use minijinja::context;
use minijinja::path_loader;
use minijinja::Environment;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::models::ExecutionId;
use std::path::PathBuf;
use std::sync::Arc;

/// Initialize the template auto-reloader
pub fn init_templates() -> Arc<AutoReloader> {
  Arc::new(AutoReloader::new(move |notifier| {
    let mut env = Environment::new();

    // Add custom filter to unescape common escape sequences
    env.add_filter("unescape", |value: String| -> String {
      value
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
    });

    if cfg!(debug_assertions) {
      tracing::info!("Running in local development mode, enabling autoreload for templates");
      let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
      env.set_loader(path_loader(&template_path));
      notifier.watch_path(&template_path, true);
    } else {
      tracing::info!("Using embedded templates");
      minijinja_embed::load_templates!(&mut env);
    }
    Ok(env)
  }))
}

/// Home page
#[tracing::instrument(skip(templates))]
pub async fn index(
  State(templates): State<Arc<AutoReloader>>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!("Rendering home page");
  let env = templates.acquire_env()?;
  let tmpl = env.get_template("index.html")?;
  let html = tmpl.render(context! { csrf_token => csrf.0 })?;
  Ok(Html(html))
}

/// List executions page
#[tracing::instrument(skip(metadata, templates))]
pub async fn list_executions(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Query(query): Query<ListExecutionsQuery>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  let limit = query.limit.unwrap_or(100);
  let offset = query.offset.unwrap_or(0);
  tracing::debug!(
    limit = limit,
    offset = offset,
    "Rendering executions list page"
  );

  // Fetch one extra to determine if there are more pages
  let mut executions = metadata
    .list_executions(Some(limit + 1), Some(offset))
    .await?;

  let has_next_page = executions.len() > limit;
  if has_next_page {
    executions.pop();
  }

  // Get total count for pagination info
  let total_count = metadata.count_executions().await?;
  let page = (offset / limit) + 1;

  tracing::debug!(
    count = executions.len(),
    total_count = total_count,
    page = page,
    "Retrieved executions for UI"
  );

  let env = templates.acquire_env()?;
  let tmpl = env.get_template("executions_list.html")?;

  let html = tmpl.render(context! {
      executions => executions,
      has_next_page => has_next_page,
      total_count => total_count,
      offset => offset,
      limit => limit,
      page => page,
      csrf_token => csrf.0,
  })?;

  Ok(Html(html))
}

/// Query parameters for execution detail page
#[derive(Debug, serde::Deserialize)]
pub struct ExecutionDetailQuery {
  #[serde(flatten)]
  list_query: ListObservationsQuery,
  /// Optional observation ID to display in side panel
  obs: Option<String>,
}

/// Execution detail page
#[tracing::instrument(skip(metadata, templates))]
pub async fn execution_detail(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path(id): Path<String>,
  Query(query): Query<ExecutionDetailQuery>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  tracing::debug!(execution_id = %id, "Rendering execution detail page");

  let execution_id = ExecutionId::parse(&id)?;
  let execution = metadata.get_execution(execution_id).await?;

  let limit = query.list_query.limit.unwrap_or(100);
  let offset = query.list_query.offset.unwrap_or(0);

  // Fetch observations with one extra to check for more pages
  let mut observations = metadata
    .list_observations(execution_id, Some(limit + 1), Some(offset))
    .await?;

  let has_next_page = observations.len() > limit;
  if has_next_page {
    observations.pop();
  }

  // Get total count for pagination info
  let total_count = metadata.count_observations(execution_id).await?;
  let page = (offset / limit) + 1;

  // If observation ID is provided, load the observation for the side panel
  let selected_observation = if let Some(obs_id) = query.obs {
    let observation_id = observation_tools_shared::ObservationId::parse(&obs_id)?;
    let obs_list = metadata.get_observations(&[observation_id]).await?;
    obs_list.into_iter().next()
  } else {
    None
  };

  tracing::debug!(
      observation_count = observations.len(),
      total_count = total_count,
      page = page,
      has_selected_obs = selected_observation.is_some(),
      execution_name = %execution.name,
      "Retrieved execution details for UI"
  );

  let env = templates.acquire_env()?;
  let tmpl = env.get_template("execution_detail.html")?;

  let html = tmpl.render(context! {
      execution => execution,
      observations => observations,
      has_next_page => has_next_page,
      total_count => total_count,
      offset => offset,
      limit => limit,
      page => page,
      selected_observation => selected_observation,
      display_threshold => observation_tools_shared::DISPLAY_THRESHOLD_BYTES,
      csrf_token => csrf.0,
  })?;

  Ok(Html(html))
}

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
