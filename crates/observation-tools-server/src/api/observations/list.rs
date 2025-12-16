//! List observations handler

use crate::api::observations::get::GetObservation;
use crate::api::observations::get::PayloadOrPointerResponse;
use crate::api::types::ListObservationsQuery;
use crate::api::AppError;
use crate::storage::MetadataStorage;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

/// Response for listing observations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListObservationsResponse {
  /// List of observations
  pub observations: Vec<GetObservation>,

  /// Whether there are more results available
  pub has_next_page: bool,
}

/// List observations for an execution
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ListObservationsQuery
    ),
    responses(
        (status = 200, description = "List of observations", body = ListObservationsResponse),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata))]
pub async fn list_observations(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path(execution_id): Path<String>,
  Query(query): Query<ListObservationsQuery>,
) -> Result<Json<ListObservationsResponse>, AppError> {
  let execution_id = ExecutionId::parse(&execution_id)?;
  let limit = query.limit.unwrap_or(100);
  let mut observations = metadata
    // Fetch one extra to see if there's a next page
    .list_observations(execution_id, Some(limit + 1), query.offset, None)
    .await?;
  let has_next_page = observations.len() > limit;
  if has_next_page {
    // Remove the extra record fetched
    observations.pop();
  }
  Ok(Json(ListObservationsResponse {
    observations: observations
      .into_iter()
      .map(|o| GetObservation {
        payload: PayloadOrPointerResponse::new(&o.observation, o.payload_or_pointer),
        observation: o.observation,
      })
      .collect(),
    has_next_page,
  }))
}
