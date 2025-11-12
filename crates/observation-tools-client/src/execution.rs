//! Execution handle for managing observation context

use crate::client::UploaderMessage;
use crate::error::Result;
use crate::Error;
use async_channel;
use napi_derive::napi;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::Observation;

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

    /// Send an observation
    pub(crate) fn send_observation(&self, observation: Observation) -> Result<()> {
        self.uploader_tx
            .try_send(UploaderMessage::Observations(vec![observation]))
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
        use observation_tools_shared::models::{Observation, ObservationId, Payload, SourceInfo};
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

        self.send_observation(observation)
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
