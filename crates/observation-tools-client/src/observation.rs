//! Observation builder API

use crate::client::ObservationUploadResult;
use crate::client::UploaderMessage;
use crate::context;
use crate::execution::ExecutionHandle;
use crate::observation_handle::ObservationHandle;
use crate::observation_handle::SendObservation;
use crate::Error;
use crate::ObservationWithPayload;
use napi_derive::napi;
use observation_tools_shared::LogLevel;
use observation_tools_shared::Markdown;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::Payload;
use observation_tools_shared::SourceInfo;
use serde::Serialize;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;

/// Builder for creating observations
///
/// Use the `observe!` macro or `ObservationBuilder::new()` to create a builder,
/// then chain methods to configure and send the observation.
///
/// Payload methods (`.serde()`, `.debug()`, `.payload()`) send the observation
/// immediately and return `SendObservation` for optional waiting.
#[derive(Clone)]
#[napi]
pub struct ObservationBuilder {
  name: String,
  labels: Vec<String>,
  metadata: HashMap<String, String>,
  source: Option<SourceInfo>,
  parent_span_id: Option<String>,
  observation_type: ObservationType,
  log_level: LogLevel,
  /// Custom observation ID (for testing)
  custom_id: Option<ObservationId>,
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
      observation_type: ObservationType::Payload,
      log_level: LogLevel::Info,
      custom_id: None,
    }
  }

  /// Set a custom observation ID (for testing)
  ///
  /// This allows tests to create an observation with a known ID, enabling
  /// navigation to the observation URL before the observation is uploaded.
  pub fn with_id(mut self, id: ObservationId) -> Self {
    self.custom_id = Some(id);
    self
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

  /// Serialize the value as JSON and send the observation
  ///
  /// Returns a `SendObservation` which allows you to wait for the upload
  /// or get the observation handle.
  pub fn serde<T: ?Sized + Serialize + 'static>(self, value: &T) -> SendObservation {
    if TypeId::of::<T>() == TypeId::of::<Payload>() {
      panic!("Use payload() method to set Payload directly");
    }
    let payload = Payload::json(serde_json::to_string(value).unwrap_or_default());
    self.send_observation(payload)
  }

  /// Send the observation with a custom payload
  ///
  /// Returns a `SendObservation` which allows you to wait for the upload
  /// or get the observation handle.
  pub fn payload<T: Into<Payload>>(self, value: T) -> SendObservation {
    self.send_observation(value.into())
  }

  /// Format the value using Debug and send the observation
  ///
  /// Uses `{:#?}` (pretty-printed Debug) for consistent, parseable output.
  /// The payload will have MIME type `text/x-rust-debug` which enables
  /// special parsing and rendering on the server.
  pub fn debug<T: Debug + ?Sized>(self, value: &T) -> SendObservation {
    let payload = Payload::debug(format!("{:#?}", value));
    self.send_observation(payload)
  }

  /// Serialize the value as JSON and send the observation with an explicit
  /// execution
  ///
  /// Use this when you have an execution handle but no execution context is
  /// set.
  pub fn serde_with_execution<T: ?Sized + Serialize + 'static>(
    self,
    value: &T,
    execution: &ExecutionHandle,
  ) -> SendObservation {
    if TypeId::of::<T>() == TypeId::of::<Payload>() {
      panic!("Use payload_with_execution() method to set Payload directly");
    }
    let payload = Payload::json(serde_json::to_string(value).unwrap_or_default());
    self.send_observation_with_execution(payload, execution)
  }

  /// Send the observation with a custom payload and explicit execution
  ///
  /// Use this when you have an execution handle but no execution context is
  /// set.
  pub fn payload_with_execution<T: Into<Payload>>(
    self,
    value: T,
    execution: &ExecutionHandle,
  ) -> SendObservation {
    self.send_observation_with_execution(value.into(), execution)
  }

  /// Format the value using Debug and send the observation with explicit
  /// execution
  ///
  /// Use this when you have an execution handle but no execution context is
  /// set.
  pub fn debug_with_execution<T: Debug + ?Sized>(
    self,
    value: &T,
    execution: &ExecutionHandle,
  ) -> SendObservation {
    let payload = Payload::debug(format!("{:#?}", value));
    self.send_observation_with_execution(payload, execution)
  }

  /// Internal method to build and send the observation
  fn send_observation(self, payload: Payload) -> SendObservation {
    match context::get_current_execution() {
      Some(execution) => self.send_observation_with_execution(payload, &execution),
      None => {
        log::trace!(
          "No execution context available for observation '{}'",
          self.name
        );
        SendObservation::stub(Error::NoExecutionContext)
      }
    }
  }

  /// Internal method to build and send the observation with explicit execution
  fn send_observation_with_execution(
    self,
    payload: Payload,
    execution: &ExecutionHandle,
  ) -> SendObservation {
    let observation_id = self.custom_id.unwrap_or_else(ObservationId::new);

    let handle = ObservationHandle {
      base_url: execution.base_url().to_string(),
      execution_id: execution.id(),
      observation_id,
    };

    // Auto-set parent_span_id from current tracing span if not explicitly set
    #[cfg(feature = "tracing")]
    let parent_span_id = self
      .parent_span_id
      .or_else(context::get_current_tracing_span_id);

    #[cfg(not(feature = "tracing"))]
    let parent_span_id = self.parent_span_id;

    let observation = Observation {
      id: observation_id,
      execution_id: execution.id(),
      name: self.name,
      observation_type: self.observation_type,
      log_level: self.log_level,
      labels: self.labels,
      metadata: self.metadata,
      source: self.source,
      parent_span_id,
      created_at: chrono::Utc::now(),
      mime_type: payload.mime_type.clone(),
      payload_size: payload.size,
    };

    let (uploaded_tx, uploaded_rx) = tokio::sync::watch::channel::<ObservationUploadResult>(None);

    // Log before sending so any error comes afterward
    log::info!(
      "Sending: {}/exe/{}/obs/{}",
      execution.base_url(),
      execution.id(),
      observation_id
    );

    if let Err(e) = execution
      .uploader_tx
      .try_send(UploaderMessage::Observations {
        observations: vec![ObservationWithPayload {
          observation,
          payload,
        }],
        handle: handle.clone(),
        uploaded_tx,
      })
    {
      log::error!("Failed to send observation: {}", e);
      return SendObservation::stub(Error::ChannelClosed);
    }

    SendObservation::new(handle, uploaded_rx)
  }
}

/// Intermediate NAPI type that holds a builder and payload, allowing `.send(exe)` pattern
#[napi]
pub struct ObservationBuilderWithPayloadNapi {
  builder: ObservationBuilder,
  payload: Payload,
}

#[napi]
impl ObservationBuilderWithPayloadNapi {
  /// Send the observation using the provided execution handle
  #[napi]
  pub fn send(&self, execution: &ExecutionHandle) -> SendObservation {
    self
      .builder
      .clone()
      .send_observation_with_execution(self.payload.clone(), execution)
  }
}

#[napi]
impl ObservationBuilder {
  /// Create a new observation builder with the given name
  #[napi(constructor)]
  pub fn new_napi(name: String) -> Self {
    Self::new(name)
  }

  /// Set a custom observation ID (for testing)
  ///
  /// This allows tests to create an observation with a known ID, enabling
  /// navigation to the observation URL before the observation is uploaded.
  #[napi(js_name = "withId")]
  pub fn with_id_napi(&mut self, id: String) -> napi::Result<&Self> {
    let observation_id = ObservationId::parse(&id)
      .map_err(|e| napi::Error::from_reason(format!("Invalid observation ID: {}", e)))?;
    self.custom_id = Some(observation_id);
    Ok(self)
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

  /// Set the payload as JSON data, returning a builder that can be sent with an execution
  #[napi(js_name = "jsonPayload")]
  pub fn json_payload_napi(&self, json_string: String) -> napi::Result<ObservationBuilderWithPayloadNapi> {
    let value = serde_json::from_str::<serde_json::Value>(&json_string)
      .map_err(|e| napi::Error::from_reason(format!("Invalid JSON payload: {}", e)))?;
    let payload = Payload::json(serde_json::to_string(&value).unwrap_or_default());
    Ok(ObservationBuilderWithPayloadNapi {
      builder: self.clone(),
      payload,
    })
  }

  /// Set the payload with custom data and MIME type, returning a builder that can be sent
  #[napi(js_name = "rawPayload")]
  pub fn raw_payload_napi(&self, data: String, mime_type: String) -> ObservationBuilderWithPayloadNapi {
    let payload = Payload::with_mime_type(data, mime_type);
    ObservationBuilderWithPayloadNapi {
      builder: self.clone(),
      payload,
    }
  }

  /// Set the payload as markdown content, returning a builder that can be sent
  #[napi(js_name = "markdownPayload")]
  pub fn markdown_payload_napi(&self, content: String) -> ObservationBuilderWithPayloadNapi {
    let payload: Payload = Markdown::from(content).into();
    ObservationBuilderWithPayloadNapi {
      builder: self.clone(),
      payload,
    }
  }
}
