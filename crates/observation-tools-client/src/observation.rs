//! Observation builder API

use crate::client::UploaderMessage;
use crate::context;
use crate::error::Result;
use crate::execution::ExecutionHandle;
use crate::observation_handle::ObservationHandle;
use crate::observation_handle::SendObservation;
use crate::Error;
use napi_derive::napi;
use observation_tools_shared::models::IntoPayload;
use observation_tools_shared::models::Observation;
use observation_tools_shared::models::ObservationId;
use observation_tools_shared::models::Payload;
use observation_tools_shared::models::SourceInfo;
use observation_tools_shared::IntoCustomPayload;
use observation_tools_shared::LogLevel;
use observation_tools_shared::ObservationType;
use std::collections::HashMap;

/// Builder for creating observations
#[napi]
pub struct ObservationBuilder {
  name: String,
  labels: Vec<String>,
  metadata: HashMap<String, String>,
  source: Option<SourceInfo>,
  parent_span_id: Option<String>,
  payload: Option<Payload>,
  observation_type: ObservationType,
  log_level: LogLevel,
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
      observation_type: ObservationType::Payload,
      log_level: LogLevel::Info,
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

  /// Set the observation type
  pub fn observation_type(mut self, observation_type: ObservationType) -> Self {
    self.observation_type = observation_type;
    self
  }

  /// Set the log level
  pub fn log_level(mut self, log_level: LogLevel) -> Self {
    self.log_level = log_level;
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
    self.build_with_execution(&execution)
  }

  /// Build and send the observation using an explicit execution handle
  pub fn build_with_execution(self, execution: &ExecutionHandle) -> Result<SendObservation> {
    let observation_id = ObservationId::new();
    let observation = Observation {
      id: observation_id,
      execution_id: execution.id(),
      name: self.name,
      observation_type: self.observation_type,
      log_level: self.log_level,
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
    Ok(SendObservation::new(
      ObservationHandle {
        base_url: execution.base_url().to_string(),
        execution_id: execution.id(),
        observation_id,
      },
      uploaded_rx,
    ))
  }
}

#[napi]
impl ObservationBuilder {
  /// Create a new observation builder with the given name
  #[napi(constructor)]
  pub fn new_napi(name: String) -> Self {
    Self {
      name,
      labels: Vec::new(),
      metadata: HashMap::new(),
      source: None,
      parent_span_id: None,
      payload: None,
      observation_type: ObservationType::Payload,
      log_level: LogLevel::Info,
    }
  }

  /// Add a label to the observation
  #[napi(js_name = "label")]
  pub fn label_napi(&mut self, label: String) -> &Self {
    self.labels.push(label);
    self
  }

  /// Add metadata to the observation
  #[napi(js_name = "metadata")]
  pub fn metadata_napi(&mut self, key: String, value: String) -> &Self {
    self.metadata.insert(key, value);
    self
  }

  /// Set the source info for the observation
  #[napi(js_name = "source")]
  pub fn source_napi(&mut self, file: String, line: u32) -> &Self {
    self.source = Some(SourceInfo {
      file,
      line,
      column: None,
    });
    self
  }

  /// Set the payload as JSON data
  #[napi(js_name = "jsonPayload")]
  pub fn json_payload_napi(&mut self, json_string: String) -> napi::Result<&Self> {
    serde_json::from_str::<serde_json::Value>(&json_string)
      .map_err(|e| napi::Error::from_reason(format!("Invalid JSON payload: {}", e)))?;

    self.payload = Some(Payload::json(json_string));
    Ok(self)
  }

  /// Set the payload with custom data and MIME type
  #[napi(js_name = "rawPayload")]
  pub fn raw_payload_napi(&mut self, data: String, mime_type: String) -> &Self {
    self.payload = Some(Payload::with_mime_type(data, mime_type));
    self
  }

  /// Set the payload as markdown content
  #[napi(js_name = "markdownPayload")]
  pub fn markdown_payload_napi(&mut self, content: String) -> &Self {
    self.payload = Some(Payload::with_mime_type(content, "text/markdown"));
    self
  }

  /// Build and send the observation
  ///
  /// Returns a SendObservation which allows you to wait for the upload to
  /// complete or get the ObservationHandle immediately.
  #[napi]
  pub fn send(&mut self, execution: &ExecutionHandle) -> napi::Result<SendObservation> {
    let observation_id = ObservationId::new();
    let observation = Observation {
      id: observation_id,
      execution_id: execution.id(),
      name: self.name.clone(),
      observation_type: self.observation_type,
      log_level: self.log_level,
      labels: std::mem::take(&mut self.labels),
      metadata: std::mem::take(&mut self.metadata),
      source: self.source.take(),
      parent_span_id: self.parent_span_id.take(),
      payload: self
        .payload
        .take()
        .ok_or_else(|| napi::Error::from_reason("Payload is required"))?,
      created_at: chrono::Utc::now(),
    };

    let (uploaded_tx, uploaded_rx) = tokio::sync::oneshot::channel();

    log::info!(
      "Sending: {}/exe/{}/obs/{}",
      execution.base_url(),
      execution.id(),
      observation_id
    );

    execution
      .uploader_tx
      .try_send(crate::client::UploaderMessage::Observations {
        observations: vec![observation],
        uploaded_tx,
      })
      .map_err(|_| napi::Error::from_reason("Channel closed"))?;

    Ok(SendObservation::new(
      ObservationHandle {
        base_url: execution.base_url().to_string(),
        execution_id: execution.id(),
        observation_id,
      },
      uploaded_rx,
    ))
  }
}
