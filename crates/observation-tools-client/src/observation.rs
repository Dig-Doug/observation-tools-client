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

/// Builder for creating observations (without payload set yet)
///
/// Call `.payload()` or `.custom_payload()` to get an
/// `ObservationBuilderWithPayload` that can be built.
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
  pub fn with_id(&mut self, id: ObservationId) -> &mut Self {
    self.custom_id = Some(id);
    self
  }

  /// Add a label to the observation
  pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
    self.labels.push(label.into());
    self
  }

  /// Add multiple labels to the observation
  pub fn labels(&mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
    self.labels.extend(labels.into_iter().map(|l| l.into()));
    self
  }

  /// Add metadata to the observation
  pub fn metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
    self.metadata.insert(key.into(), value.into());
    self
  }

  /// Set the source info for the observation
  pub fn source(&mut self, file: impl Into<String>, line: u32) -> &mut Self {
    self.source = Some(SourceInfo {
      file: file.into(),
      line,
      column: None,
    });
    self
  }

  /// Set the parent span ID
  pub fn parent_span_id(&mut self, span_id: impl Into<String>) -> &mut Self {
    self.parent_span_id = Some(span_id.into());
    self
  }

  /// Set the observation type
  pub fn observation_type(&mut self, observation_type: ObservationType) -> &mut Self {
    self.observation_type = observation_type;
    self
  }

  /// Set the log level
  pub fn log_level(&mut self, log_level: LogLevel) -> &mut Self {
    self.log_level = log_level;
    self
  }

  /// Set the payload and return a builder that can be built
  pub fn serde<T: ?Sized + Serialize + 'static>(&self, value: &T) -> ObservationBuilderWithPayload {
    if TypeId::of::<T>() == TypeId::of::<Payload>() {
      panic!("Use payload() method to set Payload directly");
    }
    ObservationBuilderWithPayload {
      fields: self.clone(),
      payload: Payload::json(serde_json::to_string(value).unwrap_or_default()),
    }
  }

  pub fn payload<T: Into<Payload>>(&self, value: T) -> ObservationBuilderWithPayload {
    ObservationBuilderWithPayload {
      fields: self.clone(),
      payload: value.into(),
    }
  }

  /// Set the payload from a Debug-formatted value and return a builder that can be built
  ///
  /// Uses `{:#?}` (pretty-printed Debug) for consistent, parseable output.
  /// The payload will have MIME type `text/x-rust-debug` which enables
  /// special parsing and rendering on the server.
  pub fn debug<T: Debug + ?Sized>(&self, value: &T) -> ObservationBuilderWithPayload {
    ObservationBuilderWithPayload {
      fields: self.clone(),
      payload: Payload::debug(format!("{:#?}", value)),
    }
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
    Ok(self.with_id(observation_id))
  }

  /// Add a label to the observation
  #[napi(js_name = "label")]
  pub fn label_napi(&mut self, label: String) -> &Self {
    self.label(label)
  }

  /// Add metadata to the observation
  #[napi(js_name = "metadata")]
  pub fn metadata_napi(&mut self, key: String, value: String) -> &Self {
    self.metadata(key, value)
  }

  /// Set the source info for the observation
  #[napi(js_name = "source")]
  pub fn source_napi(&mut self, file: String, line: u32) -> &Self {
    self.source(file, line)
  }

  /// Set the payload as JSON data
  #[napi(js_name = "jsonPayload")]
  pub fn json_payload_napi(
    &self,
    json_string: String,
  ) -> napi::Result<ObservationBuilderWithPayload> {
    let value = serde_json::from_str::<serde_json::Value>(&json_string)
      .map_err(|e| napi::Error::from_reason(format!("Invalid JSON payload: {}", e)))?;
    Ok(self.serde(&value))
  }

  /// Set the payload with custom data and MIME type
  #[napi(js_name = "rawPayload")]
  pub fn raw_payload_napi(&self, data: String, mime_type: String) -> ObservationBuilderWithPayload {
    self.serde(&Payload::with_mime_type(data, mime_type))
  }

  /// Set the payload as markdown content
  #[napi(js_name = "markdownPayload")]
  pub fn markdown_payload_napi(&self, content: String) -> ObservationBuilderWithPayload {
    self.payload(Markdown::from(content))
  }
}

/// Builder for creating observations (with payload set)
///
/// This struct is returned by `ObservationBuilder::payload()` and
/// `ObservationBuilder::custom_payload()`. It has the `build()` methods
/// since a payload is required.
#[napi]
pub struct ObservationBuilderWithPayload {
  fields: ObservationBuilder,
  payload: Payload,
}

impl ObservationBuilderWithPayload {
  /// Build and send the observation using the current execution context
  ///
  /// Returns a `SendObservation` which allows you to wait for the observation
  /// to be uploaded before proceeding, or to get the observation ID
  /// immediately.
  ///
  /// If no execution context is available, logs an error and returns a stub
  /// SendObservation that will fail on `wait_for_upload()`.
  pub fn build(self) -> SendObservation {
    match context::get_current_execution() {
      Some(execution) => self.build_with_execution(&execution),
      None => {
        log::trace!(
          "No execution context available for observation '{}'",
          self.fields.name
        );
        SendObservation::stub(Error::NoExecutionContext)
      }
    }
  }

  /// Build and send the observation using an explicit execution handle
  ///
  /// Returns a `SendObservation` which allows you to wait for the observation
  /// to be uploaded. If sending fails, returns a stub that will fail on
  /// `wait_for_upload()`.
  pub fn build_with_execution(self, execution: &ExecutionHandle) -> SendObservation {
    let observation_id = self.fields.custom_id.unwrap_or_else(ObservationId::new);

    let handle = ObservationHandle {
      base_url: execution.base_url().to_string(),
      execution_id: execution.id(),
      observation_id,
    };

    // Auto-set parent_span_id from current tracing span if not explicitly set
    #[cfg(feature = "tracing")]
    let parent_span_id = self
      .fields
      .parent_span_id
      .or_else(context::get_current_tracing_span_id);

    #[cfg(not(feature = "tracing"))]
    let parent_span_id = self.fields.parent_span_id;

    let observation = Observation {
      id: observation_id,
      execution_id: execution.id(),
      name: self.fields.name,
      observation_type: self.fields.observation_type,
      log_level: self.fields.log_level,
      labels: self.fields.labels,
      metadata: self.fields.metadata,
      source: self.fields.source,
      parent_span_id,
      created_at: chrono::Utc::now(),
      mime_type: self.payload.mime_type.clone(),
      payload_size: self.payload.size,
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
          payload: self.payload,
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

#[napi]
impl ObservationBuilderWithPayload {
  /// Build and send the observation
  ///
  /// Returns a SendObservation which allows you to wait for the upload to
  /// complete or get the ObservationHandle immediately.
  ///
  /// If sending fails, returns a stub that will fail on `wait_for_upload()`.
  #[napi]
  pub fn send(&self, execution: &ExecutionHandle) -> SendObservation {
    let with_payload = ObservationBuilderWithPayload {
      fields: self.fields.clone(),
      payload: self.payload.clone(),
    };
    with_payload.build_with_execution(execution)
  }
}
