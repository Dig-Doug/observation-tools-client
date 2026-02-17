//! Get observation handler

use crate::api::AppError;
use crate::storage::MetadataStorage;
use crate::storage::ObservationWithPayloads;
use crate::storage::PayloadData;
use crate::storage::StoredPayload;
use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use observation_tools_shared::PayloadId;
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
  pub payloads: Vec<GetPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetPayload {
  pub id: PayloadId,
  pub name: String,
  pub mime_type: String,
  pub size: usize,
  pub data: PayloadOrPointerResponse,
}

impl GetObservation {
  pub fn new(obs: ObservationWithPayloads) -> GetObservation {
    let exec_id = obs.observation.execution_id;
    let obs_id = obs.observation.id;
    let payloads = obs
      .payloads
      .into_iter()
      .map(|p| GetPayload {
        id: p.id,
        name: p.name.clone(),
        mime_type: p.mime_type.clone(),
        size: p.size,
        data: PayloadOrPointerResponse::from_stored_payload(p, exec_id, obs_id),
      })
      .collect();
    GetObservation {
      observation: obs.observation,
      payloads,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PayloadOrPointerResponse {
  Text(String),
  Json(serde_json::Value),
  Markdown { raw: String },
  InlineBinary(Vec<u8>),
  Pointer { url: String },
}

impl PayloadOrPointerResponse {
  pub fn from_stored_payload(
    payload: StoredPayload,
    exec_id: ExecutionId,
    obs_id: ObservationId,
  ) -> Self {
    let data = match payload.data {
      PayloadData::Inline(data) => data,
      PayloadData::Blob => {
        return PayloadOrPointerResponse::Pointer {
          url: format!(
            "/api/exe/{}/obs/{}/payload/{}/content",
            exec_id, obs_id, payload.id
          ),
        };
      }
    };

    if payload.mime_type.starts_with("application/json") {
      if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&data) {
        return PayloadOrPointerResponse::Json(json_value);
      }
    }
    if payload.mime_type.starts_with("text/x-rust-debug") {
      if let Ok(text) = String::from_utf8(data.clone()) {
        let parsed = crate::debug_parser::parse_debug_to_json(&text);
        return PayloadOrPointerResponse::Json(parsed);
      }
    }
    if payload.mime_type.starts_with("text/plain") {
      if let Ok(text) = String::from_utf8(data.clone()) {
        return PayloadOrPointerResponse::Text(text);
      }
    }
    if payload.mime_type.starts_with("text/markdown") {
      if let Ok(text) = String::from_utf8(data.clone()) {
        return PayloadOrPointerResponse::Markdown { raw: text };
      }
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
  let observation = metadata.get_observation(observation_id).await?;
  Ok(Json(GetObservationResponse {
    observation: GetObservation::new(observation),
  }))
}
