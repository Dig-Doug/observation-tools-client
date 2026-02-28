//! Observation handle types

use crate::client::ObservationUploadResult;
use crate::error::Result;
use crate::Error;
use napi_derive::napi;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::ObservationId;

#[napi]
pub struct SendObservation {
  pub(crate) handle: ObservationHandle,
  pub(crate) uploaded_rx: Option<tokio::sync::watch::Receiver<ObservationUploadResult>>,
  /// Error that occurred during creation (for stub observations)
  pub(crate) creation_error: Option<Error>,
}

impl SendObservation {
  pub(crate) fn new(
    handle: ObservationHandle,
    uploaded_rx: tokio::sync::watch::Receiver<ObservationUploadResult>,
  ) -> Self {
    Self {
      handle,
      uploaded_rx: Some(uploaded_rx),
      creation_error: None,
    }
  }

  /// Create a stub SendObservation that will fail on wait_for_upload()
  ///
  /// This is used when observation creation fails (e.g., no execution context).
  /// The stub allows callers to ignore errors at creation time but still
  /// detect failures when explicitly waiting.
  pub(crate) fn stub(error: Error) -> Self {
    Self {
      handle: ObservationHandle::placeholder(),
      uploaded_rx: None,
      creation_error: Some(error),
    }
  }

  pub async fn wait_for_upload(&mut self) -> Result<ObservationHandle> {
    // Return creation error if present
    if let Some(_err) = &self.creation_error {
      return Err(Error::CreationError);
    }

    // Get receiver (must be mutable for borrow_and_update and changed)
    let rx = self.uploaded_rx.as_mut().ok_or(Error::ChannelClosed)?;

    // Wait for value to change from None to Some
    loop {
      {
        let value = rx.borrow_and_update();
        match &*value {
          Some(Ok(handle)) => return Ok(handle.clone()),
          Some(Err(error_msg)) => return Err(Error::UploadFailed(error_msg.clone())),
          None => {}
        }
      }
      rx.changed().await.map_err(|_| Error::ChannelClosed)?;
    }
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

  /// Wait for the observation to be uploaded to the server
  #[napi(js_name = "waitForUpload")]
  pub async unsafe fn wait_for_upload_napi(&mut self) -> napi::Result<ObservationHandle> {
    self
      .wait_for_upload()
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

#[napi]
#[derive(Clone, Debug)]
pub struct ObservationHandle {
  pub(crate) base_url: String,
  pub(crate) observation_id: ObservationId,
  pub(crate) execution_id: ExecutionId,
}

impl ObservationHandle {
  pub fn id(&self) -> &ObservationId {
    &self.observation_id
  }

  /// Create a placeholder handle for stub observations
  pub(crate) fn placeholder() -> Self {
    Self {
      base_url: String::new(),
      observation_id: ObservationId::nil(),
      execution_id: ExecutionId::nil(),
    }
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

  #[napi(getter, js_name = "id")]
  pub fn id_napi(&self) -> String {
    self.observation_id.to_string()
  }
}

impl From<SendObservation> for ObservationHandle {
  fn from(send: SendObservation) -> ObservationHandle {
    send.into_handle()
  }
}

/// Handle for multi-part observations that allows adding additional named payloads
pub struct ObservationPayloadHandle {
  handle: ObservationHandle,
  execution: crate::execution::ExecutionHandle,
}

impl ObservationPayloadHandle {
  pub(crate) fn new(handle: ObservationHandle, execution: crate::execution::ExecutionHandle) -> Self {
    Self { handle, execution }
  }

  /// Add a named payload serialized via serde
  pub fn serde<T: ?Sized + serde::Serialize>(&self, name: impl Into<String>, value: &T) -> &Self {
    let payload =
      observation_tools_shared::Payload::json(serde_json::to_string(value).unwrap_or_default());
    self.payload(name, payload)
  }

  /// Add a named payload formatted via Debug
  pub fn debug<T: std::fmt::Debug + ?Sized>(
    &self,
    name: impl Into<String>,
    value: &T,
  ) -> &Self {
    let payload = observation_tools_shared::Payload::debug(format!("{:#?}", value));
    self.payload(name, payload)
  }

  /// Add a named payload
  pub fn payload(&self, name: impl Into<String>, payload: impl Into<observation_tools_shared::Payload>) -> &Self {
    let _ = self
      .execution
      .uploader_tx
      .try_send(crate::client::UploaderMessage::Payload {
        observation_id: self.handle.observation_id,
        execution_id: self.handle.execution_id,
        payload_id: observation_tools_shared::PayloadId::new(),
        name: name.into(),
        payload: payload.into(),
      });
    self
  }

  /// Get a reference to the observation handle
  pub fn handle(&self) -> &ObservationHandle {
    &self.handle
  }

  /// Consume and return the observation handle
  pub fn into_handle(self) -> ObservationHandle {
    self.handle
  }
}
