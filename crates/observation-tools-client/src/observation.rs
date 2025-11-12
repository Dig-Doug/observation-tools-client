//! Observation builder API

use crate::context;
use crate::error::Result;
use crate::Error;
use observation_tools_shared::models::Observation;
use observation_tools_shared::models::ObservationId;
use observation_tools_shared::models::Payload;
use observation_tools_shared::models::SourceInfo;
use std::collections::HashMap;

/// Builder for creating observations
pub struct ObservationBuilder {
  name: String,
  labels: Vec<String>,
  metadata: HashMap<String, String>,
  source: Option<SourceInfo>,
  parent_span_id: Option<String>,
  payload: Option<Payload>,
}

impl ObservationBuilder {
  /// Create a new observation builder with the given name
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      labels: Vec::new(),
      metadata: HashMap::new(),
      source: None,
      parent_span_id: None,
      payload: None,
    }
  }

  /// Add a label to the observation
  pub fn label(mut self, label: impl Into<String>) -> Self {
    self.labels.push(label.into());
    self
  }

  /// Add multiple labels to the observation
  pub fn labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self.labels.extend(labels.into_iter().map(|l| l.into()));
    self
  }

  /// Add metadata to the observation
  pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.metadata.insert(key.into(), value.into());
    self
  }

  /// Set the source info for the observation
  pub fn source(mut self, file: impl Into<String>, line: u32) -> Self {
    self.source = Some(SourceInfo {
      file: file.into(),
      line,
      column: None,
    });
    self
  }

  /// Set the parent span ID
  pub fn parent_span_id(mut self, span_id: impl Into<String>) -> Self {
    self.parent_span_id = Some(span_id.into());
    self
  }

  /// Set the payload for the observation
  pub fn payload<T: serde::Serialize>(mut self, value: T) -> Self {
    // Serialize the value to JSON string
    match serde_json::to_string(&value) {
      Ok(json_string) => {
        let size = json_string.len();
        self.payload = Some(Payload {
          mime_type: "application/json".to_string(),
          data: json_string,
          size,
        });
      }
      Err(e) => {
        tracing::error!("Failed to serialize observation payload: {}", e);
      }
    }
    self
  }

  /// Build and send the observation using the current execution context
  pub fn build(self) -> Result<ObservationId> {
    let execution = context::get_current_execution().ok_or(Error::NoExecutionContext)?;

    // Require a payload
    let payload = self.payload.ok_or(Error::MissingPayload)?;

    let observation_id = ObservationId::new();
    let execution_id = execution.id();
    let base_url = execution.base_url();
    let observation = Observation {
      id: observation_id,
      execution_id,
      name: self.name.clone(),
      labels: self.labels,
      metadata: self.metadata,
      source: self.source,
      parent_span_id: self.parent_span_id,
      payload,
      created_at: chrono::Utc::now(),
    };

    execution.send_observation(observation)?;

    // Log the observation URL
    log::info!(
      "Observation '{}' created: {}/exe/{}/obs/{}",
      self.name,
      base_url,
      execution_id,
      observation_id
    );

    Ok(observation_id)
  }
}
