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
