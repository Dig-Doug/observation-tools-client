//! Client for communicating with the observation-tools server

use crate::error::Result;
use crate::execution::ExecutionHandle;
use async_channel;
use log::trace;
use napi_derive::napi;
use observation_tools_shared::api::CreateExecutionRequest;
use observation_tools_shared::api::CreateObservationsRequest;
use observation_tools_shared::models::Execution;
use observation_tools_shared::models::Observation;
use std::sync::Arc;

/// Message types for the background uploader task
#[derive(Debug, Clone)]
pub(crate) enum UploaderMessage {
  Execution(Execution),
  Observations(Vec<Observation>),
  Flush,
  Shutdown,
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
}

#[napi]
impl Client {
  #[napi(js_name = "beginExecution")]
  pub fn begin_execution_wasm(&self, name: String) -> napi::Result<ExecutionHandle> {
    self.begin_execution(name).map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

impl Client {
  /// Begin a new execution
  pub fn begin_execution(&self, name: impl Into<String>) -> Result<ExecutionHandle> {
    let execution = Execution::new(name.into());
    trace!("Beginning new execution with ID {}", execution.id);
    self
      .inner
      .uploader_tx
      .try_send(UploaderMessage::Execution(execution.clone()))?;
    Ok(ExecutionHandle::new(
      execution.id,
      self.inner.uploader_tx.clone(),
      self.inner.base_url.clone(),
    ))
  }

  /// Shutdown the client and wait for pending uploads
  pub async fn shutdown(&self) -> Result<()> {
    self.inner.uploader_tx.try_send(UploaderMessage::Shutdown)?;
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
    let http_client = reqwest::Client::builder()
      .build()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let (tx, rx) = async_channel::unbounded();
    let uploader_base_url = base_url.clone();

    tokio::spawn(async move {
      uploader_task(http_client, uploader_base_url, rx).await;
    });

    // Spawn timer task for periodic flushes
    let timer_tx = tx.clone();
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
      interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
      loop {
        interval.tick().await;
        if timer_tx.send(UploaderMessage::Flush).await.is_err() {
          break; // Channel closed, stop timer
        }
      }
    });

    Ok(Client {
      inner: Arc::new(ClientInner {
        base_url,
        uploader_tx: tx,
      }),
    })
  }
}

async fn uploader_task(
  client: reqwest::Client,
  base_url: String,
  rx: async_channel::Receiver<UploaderMessage>,
) {
  let flush_observations = async |buffer: &mut Vec<Observation>| {
    if buffer.is_empty() {
      return;
    }
    if let Err(e) = upload_observations(&client, &base_url, buffer.drain(..).collect()).await {
      tracing::error!("Failed to upload observations: {}", e);
    }
  };
  let mut observation_buffer: Vec<Observation> = Vec::new();
  const BATCH_SIZE: usize = 100;
  loop {
    let msg = rx.recv().await.ok();
    match msg {
      Some(UploaderMessage::Execution(execution)) => {
        if let Err(e) = upload_execution(&client, &base_url, execution).await {
          tracing::error!("Failed to upload execution: {}", e);
        }
      }
      Some(UploaderMessage::Observations(observations)) => {
        observation_buffer.extend(observations);
        if observation_buffer.len() >= BATCH_SIZE {
          flush_observations(&mut observation_buffer).await;
        }
      }
      Some(UploaderMessage::Flush) => {
        flush_observations(&mut observation_buffer).await;
      }
      Some(UploaderMessage::Shutdown) | None => {
        flush_observations(&mut observation_buffer).await;
        break;
      }
    }
  }
}

// Async upload functions (used by both native and WASM)
async fn upload_execution(
  client: &reqwest::Client,
  base_url: &str,
  execution: Execution,
) -> Result<()> {
  let url = format!("{}/api/exe", base_url);
  let request = CreateExecutionRequest { execution };
  trace!("Uploading execution {:#?}", request);
  client
    .post(&url)
    .json(&request)
    .send()
    .await?
    .error_for_status()?;
  Ok(())
}

async fn upload_observations(
  client: &reqwest::Client,
  base_url: &str,
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
  for (execution_id, observations) in by_execution {
    let url = format!("{}/api/exe/{}/obs", base_url, execution_id);
    let request = CreateObservationsRequest { observations };

    client
      .post(&url)
      .json(&request)
      .send()
      .await?
      .error_for_status()?;
  }

  Ok(())
}
