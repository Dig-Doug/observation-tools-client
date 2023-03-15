use crate::token_generator::TokenGenerator;
use crate::upload_artifact_task::{UploadArtifactTask, UploadArtifactTaskPayload};
use crate::util::{encode_id_proto, new_uuid_proto, time_now, ClientError, GenericError};
use crate::PublicArtifactId;
use async_channel::{Receiver, RecvError, Sender};
use futures::TryFutureExt;
use protobuf::Message;
use reqwest::multipart::Part;
#[cfg(feature = "tokio")]
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{debug, error, trace};

#[derive(Clone)]
pub(crate) struct TaskHandler {
    pub host: String,
    pub token_generator: TokenGenerator,
    pub client: reqwest::Client,
    pub receive_task_channel: Receiver<UploadArtifactTask>,
    pub send_shutdown_channel: Sender<()>,
}

impl TaskHandler {
    pub async fn run(&self) {
        trace!("Starting receive task");

        while !self.receive_task_channel.is_closed() || !self.receive_task_channel.is_empty() {
            let task = self.receive_task_channel.recv().await;
            match task {
                Ok(t) => {
                    let result = self.handle_upload_artifact_task(&t).await;
                    if let Err(e) = result {
                        error!("Failed to upload artifact: {}", e);
                    }
                }
                Err(_) => {
                    panic!("Failed to receive task");
                }
            }
        }

        trace!("Receive task stopped");
    }

    pub(crate) async fn handle_upload_artifact_task(
        &self,
        task: &UploadArtifactTask,
    ) -> Result<PublicArtifactId, ClientError> {
        trace!("Handling artifact: {:?}", task);

        let req_b64 = base64::encode(task.request.write_to_bytes().unwrap());
        let mut form = reqwest::multipart::Form::new().text("request", req_b64);
        if let Some(payload) = task.payload.as_ref() {
            let part = match payload {
                #[cfg(feature = "files")]
                UploadArtifactTaskPayload::File(f) => {
                    let file = tokio::fs::File::open(f).await?;
                    Part::stream(reqwest::Body::wrap_stream(FramedRead::new(
                        file,
                        BytesCodec::new(),
                    )))
                }
                UploadArtifactTaskPayload::Bytes(bytes) => Part::bytes(bytes.clone()),
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
            .await
            .map_err(|e| ClientError::GenericError {
                message: e.to_string(),
            })?;

        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let response_body = response
                .text()
                .await
                .unwrap_or_else(|_| "No body".to_string());
            Err(ClientError::ServerError {
                status_code: status.as_u16(),
                response: response_body,
            })
        } else {
            Ok(())
        }
    }
}

impl Drop for TaskHandler {
    fn drop(&mut self) {
        trace!("Task handler dropped");
    }
}
