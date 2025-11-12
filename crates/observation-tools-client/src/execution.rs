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
        self.execution_id.0.to_string()
    }
}
