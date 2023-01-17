use std::cell::RefCell;
use crate::base_artifact_uploader::ContextBehavior;
use crate::base_artifact_uploader::{BaseArtifactUploaderBuilder};
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};
use crate::{RunStageUploader, RunUploader};
use artifacts_api_rust_proto::{ArtifactGroupUploaderData, CreateArtifactRequest, CreateRunRequest, CreateRunResponse, StructuredData};
use base64::decode;
use log::{debug, trace};
use protobuf::{parse_from_bytes, Message};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use async_channel::{Receiver, RecvError, Sender};
use futures::TryFutureExt;
use reqwest::multipart::Part;
use reqwest::{RequestBuilder, Response};
use tempfile::{NamedTempFile, TempDir};
#[cfg(feature = "tokio")]
use tokio::{
  runtime::{Handle, Runtime},
  sync::mpsc,
  task::JoinHandle
};
#[cfg(feature = "tokio")]
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::client::{TokenGenerator, };
use crate::upload_artifact_task::{UploadArtifactTask, UploadArtifactTaskPayload};
use crate::util::{ClientError, encode_id_proto, GenericError, new_uuid_proto, time_now};

#[derive(Clone)]
pub(crate) struct TaskHandler {
  pub host: String,
  pub token_generator: TokenGenerator,
  pub client: reqwest::Client,
  pub receive_task_channel: Receiver<UploadArtifactTask>,
  pub send_shutdown_channel: Sender<()>,
}

impl TaskHandler {
  pub async fn run(self) {
    trace!("Starting receive task");

    while !self.receive_task_channel.is_closed() || !self.receive_task_channel.is_empty() {
      let task = self.receive_task_channel.recv().await;
      match task {
        Ok(t) => {
          let result = self.handle_upload_artifact_task(&t).await;
          if let Err(e) = result {
            debug!("Failed to upload artifact: {}", e);
          }
        }
        Err(_) => {
          panic!("Failed to receive task");
        }
      }
    }

    trace!("Receive task stopped");
  }

  async fn handle_upload_artifact_task(&self, task: &UploadArtifactTask) -> Result<(), GenericError> {
    trace!("Handling artifact: {:?}", task);

    let req_b64 = base64::encode(task.request.write_to_bytes().unwrap());
    let mut form = reqwest::multipart::Form::new().text("request", req_b64);
    if let Some(payload) = task.payload.as_ref() {
      let part = match payload {
        #[cfg(feature = "files")]
        UploadArtifactTaskPayload::File(f) => {
          let file = tokio::fs::File::open(f).await?;
          Part::stream(reqwest::Body::wrap_stream(FramedRead::new(file, BytesCodec::new())))
        }
        UploadArtifactTaskPayload::Bytes(bytes) => {
          Part::bytes(bytes.clone())
        }
      };
      form = form.part("raw_data", part);
    }

    let token = self.token_generator.token().await?;
    let response = self
      .client
      .post(format!("{}/create-artifact", self.host))
      .bearer_auth(token)
      .multipart(form)
      .send()
      .await?;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
      let response_body = response.text().await.unwrap_or_else(|_| "No body".to_string());
      Err(ClientError::ServerError {
        status_code: status.as_u16(),
        response: response_body,
      }.into())
    } else {
      Ok(())
    }
  }
}

