//! Execution handle for managing observation context

use crate::client::UploaderMessage;
use crate::error::Result;
use crate::Error;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::models::Observation;
use tokio::sync::mpsc;

/// Handle to an execution that can be used to send observations
#[derive(Clone)]
pub struct ExecutionHandle {
    pub(crate) execution_id: ExecutionId,
    pub(crate) uploader_tx: mpsc::UnboundedSender<UploaderMessage>,
    pub(crate) base_url: String,
}

impl ExecutionHandle {
    pub(crate) fn new(
        execution_id: ExecutionId,
        uploader_tx: mpsc::UnboundedSender<UploaderMessage>,
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
            .send(UploaderMessage::Observations(vec![observation]))
            .map_err(|_| Error::ChannelClosed)
    }
}
