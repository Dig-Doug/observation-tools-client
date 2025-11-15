//! Client for communicating with the observation-tools server

use crate::error::Result;
use crate::execution::{BeginExecution, ExecutionHandle};
use async_channel;
use log::{error, info, trace};
use napi_derive::napi;
use observation_tools_shared::models::Execution;
use observation_tools_shared::models::Observation;
use std::sync::Arc;

// Re-export constants from shared crate for convenience
pub use observation_tools_shared::BATCH_SIZE;
pub use observation_tools_shared::BLOB_THRESHOLD_BYTES;

/// Message types for the background uploader task
pub(crate) enum UploaderMessage {
  Execution {
    execution: Execution,
    uploaded_tx: tokio::sync::oneshot::Sender<()>,
  },
  Observations {
    observations: Vec<Observation>,
    uploaded_tx: tokio::sync::oneshot::Sender<()>,
  },
  Flush,
  Shutdown,
}

// Manual Debug implementation since oneshot::Sender doesn't implement Debug
impl std::fmt::Debug for UploaderMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Execution { execution, .. } => f
        .debug_struct("Execution")
        .field("execution", execution)
        .finish(),
      Self::Observations { observations, .. } => f
        .debug_struct("Observations")
        .field("observations", observations)
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

#[napi]
impl Client {
  #[napi(js_name = "beginExecution")]
  pub fn begin_execution_wasm(&self, name: String) -> napi::Result<ExecutionHandle> {
    self
      .begin_execution(name)
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
    trace!("Beginning new execution with ID {}", execution.id);
    let (uploaded_tx, uploaded_rx) = tokio::sync::oneshot::channel();
    self
      .inner
      .uploader_tx
      .try_send(UploaderMessage::Execution {
        execution: execution.clone(),
        uploaded_tx,
      })?;
    let handle = ExecutionHandle::new(
      execution.id,
      self.inner.uploader_tx.clone(),
      self.inner.base_url.clone(),
    );
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
}

impl Default for ClientBuilder {
  fn default() -> Self {
    Self { base_url: None }
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
}

impl ClientBuilder {
  /// Set the base URL for the server
  pub fn base_url(mut self, url: impl Into<String>) -> Self {
    self.base_url = Some(url.into());
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
    let api_client = crate::server_client::create_client(&uploader_base_url)?;
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
  let flush_observations =
    async |buffer: &mut Vec<Observation>, senders: &mut Vec<tokio::sync::oneshot::Sender<()>>| {
      if buffer.is_empty() {
        return;
      }
      let result = upload_observations(&api_client, buffer.drain(..).collect()).await;
      match result {
        Ok(()) => {
          // Signal all senders that observations were uploaded
          for sender in senders.drain(..) {
            let _ = sender.send(());
          }
        }
        Err(e) => {
          error!("Failed to upload observations: {}", e);
          // Clear senders on error (they won't receive notification)
          senders.clear();
        }
      }
    };
  let mut observation_buffer: Vec<Observation> = Vec::new();
  let mut sender_buffer: Vec<tokio::sync::oneshot::Sender<()>> = Vec::new();
  loop {
    let msg = rx.recv().await.ok();
    match msg {
      Some(UploaderMessage::Execution {
        execution,
        uploaded_tx,
      }) => {
        let result = upload_execution(&api_client, execution).await;
        match result {
          Ok(()) => {
            // Signal successful upload
            let _ = uploaded_tx.send(());
          }
          Err(e) => {
            error!("Failed to upload execution: {}", e);
          }
        }
      }
      Some(UploaderMessage::Observations {
        observations,
        uploaded_tx,
      }) => {
        observation_buffer.extend(observations);
        sender_buffer.push(uploaded_tx);
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
  observations: Vec<Observation>,
) -> Result<()> {
  if observations.is_empty() {
    return Ok(());
  }

  // Group by execution_id
  let mut by_execution: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
  for obs in observations {
    by_execution.entry(obs.execution_id).or_default().push(obs);
  }

  // Upload each batch
  for (execution_id, mut observations) in by_execution {
    // Check each observation's payload size and upload large payloads as blobs
    for obs in &mut observations {
      trace!(
        "Processing observation {} with payload size {} bytes",
        obs.id,
        obs.payload.size
      );

      if obs.payload.size >= BLOB_THRESHOLD_BYTES && !obs.payload.data.is_empty() {
        trace!(
          "Uploading large payload ({} bytes) for observation {} as blob",
          obs.payload.size,
          obs.id
        );

        // Upload the payload data as a blob
        let blob_data = obs.payload.data.as_bytes().to_vec();
        if let Err(e) = client
          .upload_observation_blob(&execution_id.to_string(), &obs.id.to_string(), blob_data)
          .await
        {
          error!(
            "Failed to upload blob for observation {}: {}",
            obs.id, e
          );
          return Err(crate::error::Error::Config(format!(
            "Failed to upload blob for observation {}: {}",
            obs.id, e
          )));
        }

        // Clear the payload data since it's now stored as a blob
        obs.payload.data = String::new();

        trace!(
          "Successfully uploaded blob for observation {}, payload.data now empty",
          obs.id
        );
      } else {
        trace!(
          "Observation {} has small payload ({} bytes), keeping data inline",
          obs.id,
          obs.payload.size
        );
      }
    }

    // Convert from shared type to OpenAPI type via serde
    let observations_json = serde_json::to_value(&observations)?;
    let openapi_observations: Vec<crate::server_client::types::Observation> =
      serde_json::from_value(observations_json)?;

    client
      .create_observations()
      .execution_id(execution_id.to_string())
      .body_map(|b| b.observations(openapi_observations))
      .send()
      .await
      .map_err(|e| crate::error::Error::Config(e.to_string()))?;
  }

  Ok(())
}
