//! Web UI handlers

use crate::api::types::ListExecutionsQuery;
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
use minijinja::Value;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::ObservationType;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;

fn items_filter(value: Value) -> Value {
  if value.as_object().is_some() {
    let mut items = Vec::new();
    let Ok(values) = value.try_iter() else {
      error!("Failed to iterate over items in items filter");
      return Value::from(Vec::<Value>::new());
    };
    for key in values {
      if let Ok(val) = value.get_item(&key) {
        items.push(Value::from(vec![
          Value::from(key.as_str().unwrap_or("")),
          val,
        ]));
      }
    }
    Value::from(items)
  } else {
    Value::from(Vec::<Value>::new())
  }
}

fn render_markdown(value: String) -> String {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_FOOTNOTES);
  options.insert(Options::ENABLE_TASKLISTS);

  let parser = Parser::new_ext(&value, options);
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, parser);

  ammonia::clean(&html_output)
}

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

    // Add items filter to convert maps to iterable key-value pairs
    env.add_filter("items", items_filter);

    // Add render_markdown filter to convert markdown to sanitized HTML
    env.add_filter("render_markdown", render_markdown);

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
  execution_detail_view(metadata, templates, id, query, csrf, ExecutionView::Log).await
}

/// Execution detail page - Payload view (shows only payload observations)
#[tracing::instrument(skip(metadata, templates))]
pub async fn execution_detail_payload(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  State(templates): State<Arc<AutoReloader>>,
  Path(id): Path<String>,
  Query(query): Query<ExecutionDetailQuery>,
  csrf: CsrfToken,
) -> Result<Html<String>, AppError> {
  execution_detail_view(metadata, templates, id, query, csrf, ExecutionView::Payload).await
}

/// Shared implementation for execution detail views
async fn execution_detail_view(
  metadata: Arc<dyn MetadataStorage>,
  templates: Arc<AutoReloader>,
  id: String,
  query: ExecutionDetailQuery,
  csrf: CsrfToken,
  view: ExecutionView,
) -> Result<Html<String>, AppError> {
  tracing::debug!(execution_id = %id, ?view, "Rendering execution detail page");

  let execution_id = ExecutionId::parse(&id)?;

  // Try to get the execution, but handle not found gracefully
  let execution = match metadata.get_execution(execution_id).await {
    Ok(execution) => Some(execution),
    Err(crate::storage::StorageError::NotFound(_)) => None,
    Err(e) => return Err(e.into()),
  };

  let limit = query.limit.unwrap_or(100);
  let offset = query.offset.unwrap_or(0);

  // Determine observation type filter based on view
  let observation_type_filter = match view {
    ExecutionView::Log => None,
    ExecutionView::Payload => Some(ObservationType::Payload),
  };

  // Only fetch observations if execution exists
  let (observations, has_next_page, total_count, page) = if execution.is_some() {
    // Get total count with filter
    let total_count = metadata
      .count_observations(execution_id, observation_type_filter)
      .await?;

    // Fetch paginated observations with filter (fetch one extra to check for next
    // page)
    let mut observations = metadata
      .list_observations(
        execution_id,
        Some(limit + 1),
        Some(offset),
        observation_type_filter,
      )
      .await?;

    let has_next_page = observations.len() > limit;
    if has_next_page {
      observations.pop();
    }

    let page = (offset / limit) + 1;

    (observations, has_next_page, total_count, page)
  } else {
    (Vec::new(), false, 0, 1)
  };

  // If observation ID is provided, load the observation for the side panel
  let selected_observation = if let Some(obs_id) = &query.obs {
    let observation_id = observation_tools_shared::ObservationId::parse(obs_id)?;
    let obs_list = metadata.get_observations(&[observation_id]).await?;
    obs_list.into_iter().next()
  } else {
    None
  };

  if let Some(ref exec) = execution {
    tracing::debug!(
        observation_count = observations.len(),
        total_count = total_count,
        page = page,
        has_selected_obs = selected_observation.is_some(),
        execution_name = %exec.name,
        "Retrieved execution details for UI"
    );
  } else {
    tracing::debug!(execution_id = %id, "Execution not found, rendering waiting page");
  }

  let env = templates.acquire_env()?;
  let tmpl = env.get_template("execution_detail.html")?;

  let (view_name, base_path) = match view {
    ExecutionView::Log => ("log", format!("/exe/{}", id)),
    ExecutionView::Payload => ("payload", format!("/exe/{}/payload", id)),
  };

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
      view => view_name,
      base_path => base_path,
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
