//! Observation handle types

use crate::error::Result;
use crate::Error;
use napi_derive::napi;

#[napi]
pub struct SendObservation {
  pub(crate) handle: ObservationHandle,
  pub(crate) uploaded_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl SendObservation {
  pub(crate) fn new(
    handle: ObservationHandle,
    uploaded_rx: tokio::sync::oneshot::Receiver<()>,
  ) -> Self {
    Self {
      handle,
      uploaded_rx: Some(uploaded_rx),
    }
  }

  pub async fn wait_for_upload(self) -> Result<ObservationHandle> {
    let rx = self.uploaded_rx.ok_or(Error::ChannelClosed)?;
    rx.await.map_err(|_| Error::ChannelClosed)?;
    Ok(self.handle)
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
#[derive(Clone)]
pub struct ObservationHandle {
  pub(crate) base_url: String,
  pub(crate) observation_id: observation_tools_shared::models::ObservationId,
  pub(crate) execution_id: observation_tools_shared::models::ExecutionId,
}

impl ObservationHandle {
  pub fn id(&self) -> &observation_tools_shared::models::ObservationId {
    &self.observation_id
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
}

impl From<SendObservation> for ObservationHandle {
  fn from(send: SendObservation) -> ObservationHandle {
    send.into_handle()
  }
}
