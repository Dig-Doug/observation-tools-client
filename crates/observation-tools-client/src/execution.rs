//! Execution handle for managing observation context

use crate::client::UploaderMessage;
use crate::error::Result;
use crate::Error;
use async_channel;
use napi_derive::napi;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::Observation;

pub struct BeginExecution {
  handle: ExecutionHandle,
  uploaded_rx: tokio::sync::oneshot::Receiver<()>,
}

#[napi]
pub struct SendObservation {
  pub(crate) handle: ObservationHandle,
  pub(crate) uploaded_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl SendObservation {
  pub(crate) fn new(
    handle: ObservationHandle,
    uploaded_rx: tokio::sync::oneshot::Receiver<()>,
  ) -> Self {
    Self {
      handle,
      uploaded_rx: Some(uploaded_rx),
    }
  }

  pub async fn wait_for_upload(self) -> Result<ObservationHandle> {
    let rx = self.uploaded_rx.ok_or(Error::ChannelClosed)?;
    rx.await.map_err(|_| Error::ChannelClosed)?;
    Ok(self.handle)
  }

  pub fn handle(&self) -> &ObservationHandle {
    &self.handle
  }

  pub fn into_handle(self) -> ObservationHandle {
    self.handle
  }
}

#[napi]
impl SendObservation {
  #[napi(js_name = "handle")]
  pub fn handle_napi(&self) -> ObservationHandle {
    self.handle.clone()
  }
}

#[napi]
#[derive(Clone)]
pub struct ObservationHandle {
  pub(crate) base_url: String,
  pub(crate) observation_id: observation_tools_shared::models::ObservationId,
  pub(crate) execution_id: observation_tools_shared::models::ExecutionId,
}

impl ObservationHandle {
  pub fn id(&self) -> &observation_tools_shared::models::ObservationId {
    &self.observation_id
  }
}

#[napi]
impl ObservationHandle {
  #[napi(getter)]
  pub fn url(&self) -> String {
    format!(
      "{}/exe/{}/obs/{}",
      self.base_url, self.execution_id, self.observation_id
    )
  }
}

impl From<SendObservation> for ObservationHandle {
  fn from(send: SendObservation) -> ObservationHandle {
    send.into_handle()
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
  pub async fn wait_for_upload(self) -> Result<ExecutionHandle> {
    self.uploaded_rx.await.map_err(|_| Error::ChannelClosed)?;
    Ok(self.handle)
  }

  pub fn handle(&self) -> &ExecutionHandle {
    &self.handle
  }

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
  /// * `metadata` - Optional metadata as an array of [key, value] pairs
  #[napi(ts_return_type = "string")]
  pub fn observe(
    &self,
    name: String,
    payload_json: String,
    labels: Option<Vec<String>>,
    source_file: Option<String>,
    source_line: Option<u32>,
    metadata: Option<Vec<Vec<String>>>,
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

    // Convert metadata from array of [key, value] pairs to HashMap
    let metadata_map = metadata
      .unwrap_or_default()
      .into_iter()
      .filter_map(|pair| {
        if pair.len() == 2 {
          Some((pair[0].clone(), pair[1].clone()))
        } else {
          None
        }
      })
      .collect::<HashMap<String, String>>();

    let observation_id = ObservationId::new();
    let observation = Observation {
      id: observation_id,
      execution_id: self.execution_id,
      name: name.clone(),
      observation_type: observation_tools_shared::ObservationType::Payload,
      log_level: observation_tools_shared::LogLevel::Info,
      labels: labels.unwrap_or_default(),
      metadata: metadata_map,
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
