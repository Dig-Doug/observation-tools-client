//! Get observation handler

use crate::api::AppError;
use crate::storage::MetadataStorage;
use crate::storage::ObservationWithPayloadPointer;
use crate::storage::PayloadOrPointer;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetObservationResponse {
  pub observation: GetObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetObservation {
  #[serde(flatten)]
  pub observation: Observation,
  pub payload: PayloadOrPointerResponse,
}

impl GetObservation {
  pub fn new(obs: ObservationWithPayloadPointer) -> GetObservation {
    GetObservation {
      payload: PayloadOrPointerResponse::new(&obs.observation, obs.payload_or_pointer),
      observation: obs.observation,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PayloadOrPointerResponse {
  Text(String),
  Json(serde_json::Value),
  InlineBinary(Vec<u8>),
  Pointer { url: String },
}

impl PayloadOrPointerResponse {
  pub fn new(obs: &Observation, payload_or_pointer: PayloadOrPointer) -> Self {
    let Some(data) = payload_or_pointer.payload else {
      return PayloadOrPointerResponse::Pointer {
        url: format!("/api/exe/{}/obs/{}/content", obs.execution_id, obs.id),
      };
    };

    match obs.mime_type.as_str() {
      "application/json" => {
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&data) {
          return PayloadOrPointerResponse::Json(json_value);
        }
      }
      "text/plain" | "text/html" | "text/csv" => {
        if let Ok(text) = String::from_utf8(data.clone()) {
          return PayloadOrPointerResponse::Text(text);
        }
      }
      _ => {}
    }

    PayloadOrPointerResponse::InlineBinary(data)
  }
}

/// Get a single observation
#[utoipa::path(
    get,
    path = "/api/exe/{execution_id}/obs/{observation_id}",
    params(
        ("execution_id" = String, Path, description = "Execution ID"),
        ("observation_id" = String, Path, description = "Observation ID")
    ),
    responses(
        (status = 200, description = "Observation details", body = GetObservationResponse),
        (status = 404, description = "Observation not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "observations"
)]
#[tracing::instrument(skip(metadata))]
pub async fn get_observation(
  State(metadata): State<Arc<dyn MetadataStorage>>,
  Path((execution_id, observation_id)): Path<(String, String)>,
) -> Result<Json<GetObservationResponse>, AppError> {
  let _execution_id = ExecutionId::parse(&execution_id)?;
  let observation_id = ObservationId::parse(&observation_id)?;
  let observations = metadata.get_observations(&[observation_id]).await?;
  let observation = observations.into_iter().next().ok_or_else(|| {
    crate::storage::StorageError::NotFound(format!("Observation {} not found", observation_id))
  })?;
  Ok(Json(GetObservationResponse {
    observation: GetObservation::new(observation),
  }))
}
