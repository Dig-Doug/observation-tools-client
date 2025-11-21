//! Observation builder API

use crate::client::UploaderMessage;
use crate::context;
use crate::error::Result;
use crate::execution::{ObservationHandle, SendObservation};
use crate::Error;
use observation_tools_shared::models::IntoPayload;
use observation_tools_shared::models::Observation;
use observation_tools_shared::models::ObservationId;
use observation_tools_shared::models::Payload;
use observation_tools_shared::models::SourceInfo;
use observation_tools_shared::IntoCustomPayload;
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
  pub fn new<T: AsRef<str>>(name: T) -> Self {
    Self {
      name: name.as_ref().to_string(),
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

  pub fn payload<T: ?Sized + IntoPayload>(mut self, value: &T) -> Self {
    self.payload = Some(value.to_payload());
    self
  }

  pub fn custom_payload<T: IntoCustomPayload>(mut self, value: &T) -> Self {
    self.payload = Some(value.to_payload());
    self
  }

  /// Build and send the observation using the current execution context
  ///
  /// Returns a `SendObservation` which allows you to wait for the observation
  /// to be uploaded before proceeding, or to get the observation ID
  /// immediately.
  pub fn build(self) -> Result<SendObservation> {
    let execution = context::get_current_execution().ok_or(Error::NoExecutionContext)?;
    let observation_id = ObservationId::new();
    let observation = Observation {
      id: observation_id,
      execution_id: execution.id(),
      name: self.name,
      labels: self.labels,
      metadata: self.metadata,
      source: self.source,
      parent_span_id: self.parent_span_id,
      payload: self.payload.ok_or(Error::MissingPayload)?,
      created_at: chrono::Utc::now(),
    };
    let (uploaded_tx, uploaded_rx) = tokio::sync::oneshot::channel();
    // Log before sending so any error comes afterward
    log::info!(
      "Sending: {}/exe/{}/obs/{}",
      execution.base_url(),
      execution.id(),
      observation_id
    );
    execution
      .uploader_tx
      .try_send(UploaderMessage::Observations {
        observations: vec![observation],
        uploaded_tx,
      })
      .map_err(|_| Error::ChannelClosed)?;
    Ok(SendObservation {
      handle: ObservationHandle {
        base_url: execution.base_url().to_string(),
        execution_id: execution.id(),
        observation_id,
      },
      uploaded_rx,
    })
  }
}
