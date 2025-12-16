//! Execution handle for managing observation context

use crate::client::ExecutionUploadResult;
use crate::client::UploaderMessage;
use crate::error::Result;
use crate::Error;
use async_channel;
use napi_derive::napi;
use observation_tools_shared::models::ExecutionId;

pub struct BeginExecution {
  handle: ExecutionHandle,
  uploaded_rx: tokio::sync::watch::Receiver<ExecutionUploadResult>,
}

impl BeginExecution {
  pub(crate) fn new(
    handle: ExecutionHandle,
    uploaded_rx: tokio::sync::watch::Receiver<ExecutionUploadResult>,
  ) -> Self {
    Self {
      handle,
      uploaded_rx,
    }
  }

  /// Wait for the execution to be uploaded to the server
  pub async fn wait_for_upload(mut self) -> Result<ExecutionHandle> {
    // Wait for value to change from None to Some
    loop {
      {
        let value = self.uploaded_rx.borrow_and_update();
        match &*value {
          Some(Ok(handle)) => return Ok(handle.clone()),
          Some(Err(error_msg)) => return Err(Error::UploadFailed(error_msg.clone())),
          None => {}
        }
      }
      self
        .uploaded_rx
        .changed()
        .await
        .map_err(|_| Error::ChannelClosed)?;
    }
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
#[derive(Clone, Debug)]
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
}
