//! Client for communicating with the observation-tools server

use crate::error::Result;
use crate::execution::BeginExecution;
use crate::execution::ExecutionHandle;
use crate::observation_handle::ObservationHandle;
use crate::ObservationWithPayload;
use async_channel;
use log::error;
use log::info;
use log::trace;
use napi_derive::napi;
use observation_tools_shared::models::Execution;
// Re-export constants from shared crate for convenience
pub use observation_tools_shared::BATCH_SIZE;
pub use observation_tools_shared::BLOB_THRESHOLD_BYTES;
use std::sync::Arc;

/// Result type for observation upload completion notifications via watch
/// channel Uses String for error since crate::Error doesn't implement Clone
pub(crate) type ObservationUploadResult = Option<std::result::Result<ObservationHandle, String>>;

/// Result type for execution upload completion notifications via watch channel
/// Uses String for error since crate::Error doesn't implement Clone
pub(crate) type ExecutionUploadResult = Option<std::result::Result<ExecutionHandle, String>>;

/// Message types for the background uploader task
pub(crate) enum UploaderMessage {
  Execution {
    execution: Execution,
    handle: ExecutionHandle,
    uploaded_tx: tokio::sync::watch::Sender<ExecutionUploadResult>,
  },
  Observations {
    observations: Vec<ObservationWithPayload>,
    handle: ObservationHandle,
    uploaded_tx: tokio::sync::watch::Sender<ObservationUploadResult>,
  },
  Flush,
  Shutdown,
}

// Manual Debug implementation since watch::Sender doesn't implement Debug
impl std::fmt::Debug for UploaderMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Execution { execution, .. } => f
        .debug_struct("Execution")
        .field("execution", execution)
        .finish(),
      Self::Observations {
        observations,
        handle,
        ..
      } => f
        .debug_struct("Observations")
        .field("observations", observations)
        .field("handle", handle)
        .finish(),
      Self::Flush => write!(f, "Flush"),
      Self::Shutdown => write!(f, "Shutdown"),
    }
  }
}

/// Client for observation-tools
#[napi]
#[derive(Clone)]
pub struct Client {
  inner: Arc<ClientInner>,
}

struct ClientInner {
  base_url: String,
  uploader_tx: async_channel::Sender<UploaderMessage>,
  shutdown_rx: std::sync::Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
  // If we create a runtime for the uploader, we hold it here to keep it alive
  _runtime: Option<Arc<tokio::runtime::Runtime>>,
}

/// Generate a new execution ID (for testing)
///
/// This allows tests to generate an execution ID before creating the execution,
/// enabling navigation to the execution URL before the execution is uploaded.
#[napi(js_name = "generateExecutionId")]
#[allow(unused)]
pub fn generate_execution_id() -> String {
  observation_tools_shared::models::ExecutionId::new().to_string()
}

/// Generate a new observation ID (for testing)
///
/// This allows tests to generate an observation ID before creating the
/// observation, enabling navigation to the observation URL before the
/// observation is uploaded.
#[napi(js_name = "generateObservationId")]
#[allow(unused)]
pub fn generate_observation_id() -> String {
  observation_tools_shared::ObservationId::new().to_string()
}

#[napi]
impl Client {
  #[napi(js_name = "beginExecution")]
  pub fn begin_execution_wasm(&self, name: String) -> napi::Result<ExecutionHandle> {
    self
      .begin_execution(name)
      .map(|begin| begin.into_handle())
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Begin a new execution with a specific ID (for testing)
  ///
  /// This allows tests to create an execution with a known ID, enabling
  /// navigation to the execution URL before the execution is uploaded.
  #[napi(js_name = "beginExecutionWithId")]
  pub fn begin_execution_with_id_wasm(
    &self,
    id: String,
    name: String,
  ) -> napi::Result<ExecutionHandle> {
    let execution_id = observation_tools_shared::models::ExecutionId::parse(&id)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let execution = Execution::with_id(execution_id, name);
    self
      .begin_execution_internal(execution)
      .map(|begin| begin.into_handle())
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

impl Client {
  /// Begin a new execution
  ///
  /// Returns a `BeginExecution` which allows you to wait for the execution
  /// to be uploaded before proceeding, or to get the handle immediately.
  pub fn begin_execution(&self, name: impl Into<String>) -> Result<BeginExecution> {
    let execution = Execution::new(name.into());
    self.begin_execution_internal(execution)
  }

  fn begin_execution_internal(&self, execution: Execution) -> Result<BeginExecution> {
    trace!("Beginning new execution with ID {}", execution.id);
    let handle = ExecutionHandle::new(
      execution.id,
      self.inner.uploader_tx.clone(),
      self.inner.base_url.clone(),
    );
    let (uploaded_tx, uploaded_rx) = tokio::sync::watch::channel(None);
    self
      .inner
      .uploader_tx
      .try_send(UploaderMessage::Execution {
        execution: execution.clone(),
        handle: handle.clone(),
        uploaded_tx,
      })?;
    Ok(BeginExecution::new(handle, uploaded_rx))
  }

  /// Shutdown the client and wait for pending uploads
  pub async fn shutdown(&self) -> Result<()> {
    self.inner.uploader_tx.try_send(UploaderMessage::Shutdown)?;
    // Wait for the uploader thread to finish
    if let Some(rx) = self.inner.shutdown_rx.lock().unwrap().take() {
      let _ = rx.await;
    }
    Ok(())
  }
}

impl Drop for ClientInner {
  fn drop(&mut self) {
    // Best effort shutdown notification
    let _ = self.uploader_tx.try_send(UploaderMessage::Shutdown);
  }
}

/// Builder for Client
#[napi]
pub struct ClientBuilder {
  base_url: Option<String>,
  api_key: Option<String>,
}

impl Default for ClientBuilder {
  fn default() -> Self {
    Self {
      base_url: None,
      api_key: None,
    }
  }
}

#[napi]
impl ClientBuilder {
  /// Create a new client builder
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the base URL for the server
  #[napi]
  pub fn set_base_url(&mut self, url: String) {
    self.base_url = Some(url);
  }

  /// Set the API key for authentication
  #[napi]
  pub fn set_api_key(&mut self, api_key: String) {
    self.api_key = Some(api_key);
  }
}

impl ClientBuilder {
  /// Set the base URL for the server
  pub fn base_url(mut self, url: impl Into<String>) -> Self {
    self.base_url = Some(url.into());
    self
  }

  /// Set the API key for authentication
  pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
    self.api_key = Some(api_key.into());
    self
  }
}

#[napi]
impl ClientBuilder {
  /// Build the client
  #[napi]
  pub fn build(&self) -> napi::Result<Client> {
    let base_url = self
      .base_url
      .clone()
      .unwrap_or_else(|| "http://localhost:3000".to_string());
    let (tx, rx) = async_channel::unbounded();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let timer_tx = tx.clone();
    let uploader_base_url = base_url.clone();
    let api_key = self.api_key.clone();
    let (handle, runtime) = match tokio::runtime::Handle::try_current() {
      Ok(handle) => (handle, None),
      Err(_) => {
        let runtime = Arc::new(
          tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?,
        );
        (runtime.handle().clone(), Some(runtime))
      }
    };
    let api_client = crate::server_client::create_client(&uploader_base_url, api_key.clone())?;
    handle.spawn(async move {
      tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
          interval.tick().await;
          if timer_tx.send(UploaderMessage::Flush).await.is_err() {
            break; // Channel closed, stop timer
          }
        }
      });
      uploader_task(api_client, rx).await;
      let _ = shutdown_tx.send(());
    });
    Ok(Client {
      inner: Arc::new(ClientInner {
        base_url,
        uploader_tx: tx,
        shutdown_rx: std::sync::Mutex::new(Some(shutdown_rx)),
        _runtime: runtime,
      }),
    })
  }
}

async fn uploader_task(
  api_client: crate::server_client::Client,
  rx: async_channel::Receiver<UploaderMessage>,
) {
  info!("Uploader task started");

  // Buffer type for observation senders: (handle, sender)
  type ObservationSender = (
    ObservationHandle,
    tokio::sync::watch::Sender<ObservationUploadResult>,
  );

  let flush_observations = async |buffer: &mut Vec<ObservationWithPayload>,
                                  senders: &mut Vec<ObservationSender>| {
    if buffer.is_empty() {
      return;
    }
    let result = upload_observations(&api_client, buffer.drain(..).collect()).await;
    match result {
      Ok(()) => {
        // Signal all senders that observations were uploaded successfully
        for (handle, sender) in senders.drain(..) {
          let _ = sender.send(Some(Ok(handle)));
        }
      }
      Err(e) => {
        let error_msg = e.to_string();
        error!("Failed to upload observations: {}", error_msg);
        // Signal all senders with the error (as String for Clone compatibility)
        for (_, sender) in senders.drain(..) {
          let _ = sender.send(Some(Err(error_msg.clone())));
        }
      }
    }
  };
  let mut observation_buffer: Vec<ObservationWithPayload> = Vec::new();
  let mut sender_buffer: Vec<ObservationSender> = Vec::new();
  loop {
    let msg = rx.recv().await.ok();
    match msg {
      Some(UploaderMessage::Execution {
        execution,
        handle,
        uploaded_tx,
      }) => {
        let result = upload_execution(&api_client, execution).await;
        match result {
          Ok(()) => {
            // Signal successful upload with handle
            let _ = uploaded_tx.send(Some(Ok(handle)));
          }
          Err(e) => {
            let error_msg = e.to_string();
            error!("Failed to upload execution: {}", error_msg);
            let _ = uploaded_tx.send(Some(Err(error_msg)));
          }
        }
      }
      Some(UploaderMessage::Observations {
        observations,
        handle,
        uploaded_tx,
      }) => {
        observation_buffer.extend(observations);
        sender_buffer.push((handle, uploaded_tx));
        if observation_buffer.len() >= BATCH_SIZE {
          flush_observations(&mut observation_buffer, &mut sender_buffer).await;
        }
      }
      Some(UploaderMessage::Flush) => {
        flush_observations(&mut observation_buffer, &mut sender_buffer).await;
      }
      Some(UploaderMessage::Shutdown) | None => {
        flush_observations(&mut observation_buffer, &mut sender_buffer).await;
        break;
      }
    }
  }
}

// Async upload functions (used by both native and WASM)
async fn upload_execution(
  client: &crate::server_client::Client,
  execution: Execution,
) -> Result<()> {
  trace!("Uploading execution");

  // Convert from shared type to OpenAPI type via serde
  let execution_json = serde_json::to_value(&execution)?;
  let openapi_execution: crate::server_client::types::Execution =
    serde_json::from_value(execution_json)?;

  client
    .create_execution()
    .body_map(|b| b.execution(openapi_execution))
    .send()
    .await
    .map_err(|e| crate::error::Error::Config(e.to_string()))?;

  Ok(())
}

async fn upload_observations(
  client: &crate::server_client::Client,
  observations: Vec<ObservationWithPayload>,
) -> Result<()> {
  if observations.is_empty() {
    return Ok(());
  }

  // Group by execution_id
  let mut by_execution: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
  for obs in observations {
    by_execution
      .entry(obs.observation.execution_id)
      .or_default()
      .push(obs);
  }

  // Upload each batch via multipart form
  for (execution_id, observations) in by_execution {
    trace!(
      "Uploading {} observations for execution {}",
      observations.len(),
      execution_id
    );

    client
      .create_observations_multipart(&execution_id.to_string(), observations)
      .await
      .map_err(|e| crate::error::Error::Config(e.to_string()))?;
  }

  Ok(())
}
