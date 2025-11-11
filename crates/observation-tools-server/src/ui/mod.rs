//! Web UI handlers

use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use minijinja::context;
use minijinja::path_loader;
use minijinja::Environment;
use minijinja_autoreload::AutoReloader;
use observation_tools_shared::api::ListExecutionsQuery;
use observation_tools_shared::api::ListObservationsQuery;
use observation_tools_shared::models::ExecutionId;
use std::path::PathBuf;
use std::sync::Arc;

/// Initialize the template auto-reloader
pub fn init_templates() -> Arc<AutoReloader> {
    Arc::new(AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
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
pub async fn index(State(templates): State<Arc<AutoReloader>>) -> impl IntoResponse {
    tracing::debug!("Rendering home page");
    let env = templates.acquire_env().unwrap();
    let tmpl = env.get_template("index.html").unwrap();
    Html(tmpl.render(context! {}).unwrap())
}

/// List executions page
#[tracing::instrument(skip(metadata, templates))]
pub async fn list_executions(
    State(metadata): State<Arc<dyn MetadataStorage>>,
    State(templates): State<Arc<AutoReloader>>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<Html<String>, AppError> {
    let limit = query.limit.unwrap_or(100);
    tracing::debug!(limit = limit, offset = ?query.offset, "Rendering executions list page");

    // Fetch one extra to determine if there are more pages
    let mut executions = metadata
        .list_executions(Some(limit + 1), query.offset)
        .await?;

    let has_next_page = executions.len() > limit;
    if has_next_page {
        executions.pop();
    }

    tracing::debug!(count = executions.len(), "Retrieved executions for UI");

    let env = templates.acquire_env().unwrap();
    let tmpl = env.get_template("executions_list.html").unwrap();

    let html = tmpl
        .render(context! {
            executions => executions,
            has_next_page => has_next_page,
        })
        .unwrap();

    Ok(Html(html))
}

/// Execution detail page
#[tracing::instrument(skip(metadata, templates))]
pub async fn execution_detail(
    State(metadata): State<Arc<dyn MetadataStorage>>,
    State(templates): State<Arc<AutoReloader>>,
    Path(id): Path<String>,
    Query(query): Query<ListObservationsQuery>,
) -> Result<Html<String>, AppError> {
    tracing::debug!(execution_id = %id, "Rendering execution detail page");

    let execution_id = ExecutionId::parse(&id)?;
    let execution = metadata.get_execution(execution_id).await?;

    let limit = query.limit.unwrap_or(100);

    // Fetch observations with one extra to check for more pages
    let mut observations = metadata
        .list_observations(execution_id, Some(limit + 1), query.offset)
        .await?;

    let has_next_page = observations.len() > limit;
    if has_next_page {
        observations.pop();
    }

    tracing::debug!(
        observation_count = observations.len(),
        execution_name = %execution.name,
        "Retrieved execution details for UI"
    );

    let env = templates.acquire_env().unwrap();
    let tmpl = env.get_template("execution_detail.html").unwrap();

    let html = tmpl
        .render(context! {
            execution => execution,
            observations => observations,
            has_next_page => has_next_page,
        })
        .unwrap();

    Ok(Html(html))
}

/// Observation detail (for the side panel)
#[tracing::instrument(skip(metadata, templates))]
pub async fn observation_detail(
    State(metadata): State<Arc<dyn MetadataStorage>>,
    State(templates): State<Arc<AutoReloader>>,
    Path((execution_id, observation_id)): Path<(String, String)>,
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

    let env = templates.acquire_env().unwrap();
    let tmpl = env.get_template("observation_detail.html").unwrap();

    let html = tmpl
        .render(context! {
            observation => observation,
        })
        .unwrap();

    Ok(Html(html))
}
