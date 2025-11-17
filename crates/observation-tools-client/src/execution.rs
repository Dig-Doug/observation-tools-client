//! Execution handle for managing observation context

use crate::client::UploaderMessage;
use crate::error::Result;
use crate::Error;
use async_channel;
use napi_derive::napi;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::Observation;

/// Result of beginning a new execution
///
/// This type is returned when you start a new execution and allows you to
/// wait for the execution to be uploaded to the server before proceeding.
pub struct BeginExecution {
  handle: ExecutionHandle,
  uploaded_rx: tokio::sync::oneshot::Receiver<()>,
}

/// Result of sending observation(s)
///
/// This type is returned when you send observations and allows you to
/// wait for the observations to be uploaded to the server before proceeding.
pub struct SendObservation {
  observation_id: observation_tools_shared::models::ObservationId,
  uploaded_rx: tokio::sync::oneshot::Receiver<()>,
}

impl SendObservation {
  pub(crate) fn new(
    observation_id: observation_tools_shared::models::ObservationId,
    uploaded_rx: tokio::sync::oneshot::Receiver<()>,
  ) -> Self {
    Self {
      observation_id,
      uploaded_rx,
    }
  }

  /// Wait for the observations to be uploaded to the server
  ///
  /// This consumes the SendObservation and returns the observation ID
  /// after the observations have been successfully uploaded.
  ///
  /// # Returns
  /// - `Ok(ObservationId)` if the observations were successfully uploaded
  /// - `Err(Error::ChannelClosed)` if the upload task failed
  pub async fn wait_for_upload(self) -> Result<observation_tools_shared::models::ObservationId> {
    self.uploaded_rx.await.map_err(|_| Error::ChannelClosed)?;
    Ok(self.observation_id)
  }

  /// Get the observation ID without waiting for upload
  pub fn observation_id(&self) -> observation_tools_shared::models::ObservationId {
    self.observation_id
  }
}

impl BeginExecution {
  pub(crate) fn new(
    handle: ExecutionHandle,
    uploaded_rx: tokio::sync::oneshot::Receiver<()>,
  ) -> Self {
    Self {
      handle,
      uploaded_rx,
    }
  }

  /// Wait for the execution to be uploaded to the server
  ///
  /// This consumes the BeginExecution and returns the ExecutionHandle
  /// after the execution has been successfully uploaded.
  ///
  /// # Returns
  /// - `Ok(ExecutionHandle)` if the execution was successfully uploaded
  /// - `Err(Error::ChannelClosed)` if the upload task failed
  pub async fn wait_for_upload(self) -> Result<ExecutionHandle> {
    self.uploaded_rx.await.map_err(|_| Error::ChannelClosed)?;
    Ok(self.handle)
  }

  /// Get a reference to the execution handle without waiting for upload
  ///
  /// This is useful if you want to start sending observations immediately
  /// without waiting for the execution creation to complete.
  pub fn handle(&self) -> &ExecutionHandle {
    &self.handle
  }

  /// Consume this and return the execution handle without waiting for upload
  ///
  /// This is useful if you don't care about waiting for the execution
  /// to be uploaded before proceeding.
  pub fn into_handle(self) -> ExecutionHandle {
    self.handle
  }
}

/// Handle to an execution that can be used to send observations
#[napi]
#[derive(Clone)]
pub struct ExecutionHandle {
  pub(crate) execution_id: ExecutionId,
  pub(crate) uploader_tx: async_channel::Sender<UploaderMessage>,
  pub(crate) base_url: String,
}

impl ExecutionHandle {
  pub(crate) fn new(
    execution_id: ExecutionId,
    uploader_tx: async_channel::Sender<UploaderMessage>,
    base_url: String,
  ) -> Self {
    Self {
      execution_id,
      uploader_tx,
      base_url,
    }
  }

  /// Get the execution ID
  pub fn id(&self) -> ExecutionId {
    self.execution_id
  }

  /// Get the base URL for the observation server
  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  /// Send an observation (internal use, doesn't wait for upload)
  pub(crate) fn send_observation(&self, observation: Observation) -> Result<()> {
    // Create a oneshot channel but drop the receiver since we don't wait
    let (uploaded_tx, _uploaded_rx) = tokio::sync::oneshot::channel();

    self
      .uploader_tx
      .try_send(UploaderMessage::Observations {
        observations: vec![observation],
        uploaded_tx,
      })
      .map_err(|_| Error::ChannelClosed)
  }

  /// Send a pre-built observation
  ///
  /// Returns a `SendObservation` which allows you to wait for the observation
  /// to be uploaded before proceeding, or to get the observation ID
  /// immediately.
  pub fn send_observation_data(&self, mut observation: Observation) -> Result<SendObservation> {
    // Ensure the observation belongs to this execution
    observation.execution_id = self.execution_id;
    let obs_id = observation.id;

    // Create a oneshot channel to signal when observations are uploaded
    let (uploaded_tx, uploaded_rx) = tokio::sync::oneshot::channel();

    self
      .uploader_tx
      .try_send(UploaderMessage::Observations {
        observations: vec![observation],
        uploaded_tx,
      })
      .map_err(|_| Error::ChannelClosed)?;

    Ok(SendObservation::new(obs_id, uploaded_rx))
  }
}

#[napi]
impl ExecutionHandle {
  /// Get the execution ID as a string
  #[napi(getter)]
  pub fn id_string(&self) -> String {
    self.execution_id.to_string()
  }

  /// Get the URL to the execution page
  #[napi(getter)]
  pub fn url(&self) -> String {
    format!("{}/exe/{}", self.base_url, self.execution_id)
  }

  /// Create and send an observation
  ///
  /// # Arguments
  /// * `name` - The name of the observation
  /// * `payload_json` - The data to observe as a JSON string
  /// * `labels` - Optional array of labels for categorization
  /// * `source_file` - Optional source file path
  /// * `source_line` - Optional source line number
  #[napi(ts_return_type = "string")]
  pub fn observe(
    &self,
    name: String,
    payload_json: String,
    labels: Option<Vec<String>>,
    source_file: Option<String>,
    source_line: Option<u32>,
  ) -> napi::Result<String> {
    use observation_tools_shared::models::Observation;
    use observation_tools_shared::models::ObservationId;
    use observation_tools_shared::models::Payload;
    use observation_tools_shared::models::SourceInfo;
    use std::collections::HashMap;

    // Validate that it's valid JSON
    serde_json::from_str::<serde_json::Value>(&payload_json)
      .map_err(|e| napi::Error::from_reason(format!("Invalid JSON payload: {}", e)))?;

    let size = payload_json.len();
    let payload_data = Payload {
      mime_type: "application/json".to_string(),
      data: payload_json,
      size,
    };

    let source = match (source_file, source_line) {
      (Some(file), Some(line)) => Some(SourceInfo {
        file,
        line,
        column: None,
      }),
      _ => None,
    };

    let observation_id = ObservationId::new();
    let observation = Observation {
      id: observation_id,
      execution_id: self.execution_id,
      name: name.clone(),
      labels: labels.unwrap_or_default(),
      metadata: HashMap::new(),
      source,
      parent_span_id: None,
      payload: payload_data,
      created_at: chrono::Utc::now(),
    };

    self
      .send_observation(observation)
      .map_err(|e| napi::Error::from_reason(format!("Failed to send observation: {}", e)))?;

    log::info!(
      "Observation '{}' created: {}/exe/{}/obs/{}",
      name,
      self.base_url,
      self.execution_id,
      observation_id
    );

    Ok(observation_id.to_string())
  }
}
