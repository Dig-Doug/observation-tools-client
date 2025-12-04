//! List executions page handler

use crate::api::types::ListExecutionsQuery;
use crate::api::AppError;
use crate::csrf::CsrfToken;
use crate::storage::MetadataStorage;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use std::sync::Arc;

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
