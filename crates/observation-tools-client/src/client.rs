//! Client for communicating with the observation-tools server

use crate::{error::Result, execution::ExecutionHandle, Error};
use observation_tools_shared::{
    api::{CreateExecutionRequest, CreateObservationsRequest},
    models::{Execution, Observation},
};
use std::sync::Arc;
use log::trace;
use tokio::sync::mpsc;

/// Message types for the background uploader task
#[derive(Debug)]
pub(crate) enum UploaderMessage {
    Execution(Execution),
    Observations(Vec<Observation>),
    Shutdown,
}

/// Client for observation-tools
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    http_client: reqwest::Client,
    base_url: String,
    uploader_tx: mpsc::UnboundedSender<UploaderMessage>,
}

impl Client {
    /// Create a new client builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Begin a new execution
    pub fn begin_execution(&self, name: impl Into<String>) -> Result<ExecutionHandle> {
        let execution = Execution::new(name.into());
        trace!("Beginning new execution with ID {}", execution.id);

        // Send to uploader
        self.inner
            .uploader_tx
            .send(UploaderMessage::Execution(execution.clone()))
            .map_err(|_| Error::ChannelClosed)?;

        Ok(ExecutionHandle::new(
            execution.id,
            self.inner.uploader_tx.clone(),
            self.inner.base_url.clone(),
        ))
    }

    /// Shutdown the client and wait for pending uploads
    pub async fn shutdown(&self) -> Result<()> {
        self.inner
            .uploader_tx
            .send(UploaderMessage::Shutdown)
            .map_err(|_| Error::ChannelClosed)?;
        Ok(())
    }
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        // Best effort shutdown notification
        let _ = self.uploader_tx.send(UploaderMessage::Shutdown);
    }
}

/// Builder for Client
pub struct ClientBuilder {
    base_url: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: None,
            http_client: None,
        }
    }
}

impl ClientBuilder {
    /// Set the base URL for the server
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set a custom HTTP client
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<Client> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| "http://localhost:3000".to_string());
        let http_client = self.http_client.unwrap_or_else(|| {
            reqwest::Client::builder()
                .build()
                .expect("Failed to create HTTP client")
        });

        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn background uploader task
        let uploader_client = http_client.clone();
        let uploader_base_url = base_url.clone();
        tokio::spawn(async move {
            uploader_task(uploader_client, uploader_base_url, rx).await;
        });

        Ok(Client {
            inner: Arc::new(ClientInner {
                http_client,
                base_url,
                uploader_tx: tx,
            }),
        })
    }
}

/// Helper function to flush observations with error logging
async fn flush_observations(
    client: &reqwest::Client,
    base_url: &str,
    observations: Vec<Observation>,
) {
    if let Err(e) = upload_observations(client, base_url, observations).await {
        tracing::error!("Failed to upload observations: {}", e);
    }
}

/// Background task that uploads observations to the server
async fn uploader_task(
    client: reqwest::Client,
    base_url: String,
    mut rx: mpsc::UnboundedReceiver<UploaderMessage>,
) {
    let mut observation_buffer: Vec<Observation> = Vec::new();
    let batch_size = 100;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Upload buffered observations after 1 second even if batch not full
                if !observation_buffer.is_empty() {
                    flush_observations(
                        &client,
                        &base_url,
                        observation_buffer.drain(..).collect(),
                    )
                    .await;
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(UploaderMessage::Execution(execution)) => {
                        // Upload execution immediately
                        if let Err(e) = upload_execution(&client, &base_url, execution).await {
                            tracing::error!("Failed to upload execution: {}", e);
                        }
                    }
                    Some(UploaderMessage::Observations(observations)) => {
                        observation_buffer.extend(observations);

                        // Upload batch if we have enough observations
                        if observation_buffer.len() >= batch_size {
                            flush_observations(
                                &client,
                                &base_url,
                                observation_buffer.drain(..).collect(),
                            )
                            .await;
                        }
                    }
                    Some(UploaderMessage::Shutdown) | None => {
                        // Flush remaining observations
                        if !observation_buffer.is_empty() {
                            flush_observations(&client, &base_url, observation_buffer).await;
                        }
                        break;
                    }
                }
            }
        }
    }
}

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
        by_execution
            .entry(obs.execution_id)
            .or_default()
            .push(obs);
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
