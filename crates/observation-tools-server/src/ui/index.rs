//! Home page handler

use crate::api::AppError;
use crate::csrf::CsrfToken;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use std::sync::Arc;

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
