use crate::generated::CreateArtifactRequest;
use crate::token_generator::TokenGenerator;
use crate::util::ClientError;
use crate::util::GenericError;
use crate::PublicArtifactId;
use crate::PublicArtifactIdTaskHandle;
use protobuf::Message;
use reqwest::multipart::Part;
use std::io::Write;
use std::sync::Arc;
use tracing::error;
use tracing::trace;

#[derive(Debug)]
pub struct TaskLoop {
    task_handler: Arc<TaskHandler>,
    params: TaskLoopParams,
    send_task_channel: async_channel::Sender<UploadArtifactTask>,
    receive_shutdown_channel: async_channel::Receiver<()>,
    #[cfg(feature = "files")]
    tmp_dir: Arc<tempfile::TempDir>,
}

#[derive(Debug)]
pub enum TaskLoopParams {
    #[cfg(feature = "tokio")]
    TokioRuntime {
        runtime: tokio::runtime::Handle,
    },
    None,
}

#[derive(Debug)]
pub struct UploadArtifactTask {
    pub request: CreateArtifactRequest,
    pub payload: Option<UploadArtifactTaskPayload>,
    pub completion_channel: async_channel::Sender<()>,
}

#[derive(Debug)]
pub enum UploadArtifactTaskPayload {
    #[cfg(feature = "files")]
    File(tempfile::NamedTempFile),
    Bytes(Vec<u8>),
}

impl TaskLoop {
    pub(crate) fn new(task_handler: Arc<TaskHandler>) -> Result<TaskLoop, GenericError> {
        let (send_task_channel, receive_task_channel) = async_channel::unbounded();
        let (send_shutdown_channel, receive_shutdown_channel) = async_channel::bounded(1);
        let task_loop = {
            let task_handler = task_handler.clone();
            async move {
                trace!("Taskloop started");

                while !receive_task_channel.is_closed() || !receive_task_channel.is_empty() {
                    let task = receive_task_channel.recv().await;
                    match task {
                        Ok(t) => {
                            let result = task_handler.handle_upload_artifact_task(&t).await;
                            if let Err(e) = result {
                                error!("Failed to upload artifact: {}", e);
                            }
                        }
                        Err(_) => {
                            error!("Failed to receive task");
                        }
                    }
                }

                trace!("Shutting down task loop");
                if let Err(e) = send_shutdown_channel.send(()).await {
                    error!("Error shutting down: {}", e);
                }
            }
        };

        #[cfg(feature = "tokio")]
        let params = {
            let runtime = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(_e) => {
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()?;
                    runtime.handle().clone()
                }
            };
            runtime.spawn(task_loop);
            TaskLoopParams::TokioRuntime { runtime }
        };
        #[cfg(not(feature = "tokio"))]
        let params = TaskLoopParams::None;

        Ok(TaskLoop {
            task_handler,
            params,
            receive_shutdown_channel,
            send_task_channel,
            #[cfg(feature = "files")]
            tmp_dir: Self::create_tmp_dir(),
        })
    }

    pub fn submit_task(
        &self,
        request: &CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let id = request
            .artifact_id
            .as_ref()
            .expect("ArtifactId missing on upload task")
            .clone();
        trace!(
            "Submitting artifact: {} raw_data_len: {}",
            id,
            raw_data.map(|d| d.len() as i64).unwrap_or(-1)
        );

        #[cfg(feature = "files")]
        let payload = {
            let tmp_file = if let Some(raw_data_slice) = raw_data {
                // TODO(doug): Consider using a spooled tempfile
                let mut tmp_file = tempfile::NamedTempFile::new_in(&*self.tmp_dir).unwrap();
                tmp_file.write_all(raw_data_slice).unwrap();
                Some(tmp_file)
            } else {
                None
            };
            tmp_file.map(|f| UploadArtifactTaskPayload::File(f))
        };
        #[cfg(not(feature = "files"))]
        let payload = raw_data.map(|bytes| UploadArtifactTaskPayload::Bytes(bytes.to_vec()));
        // TODO(doug): Do we need to make a copy of the raw data here?

        let (send_completion_channel, receive_completion_channel) = async_channel::unbounded();
        let task = UploadArtifactTask {
            request: request.clone(),
            payload,
            completion_channel: send_completion_channel,
        };

        self.send_task_channel
            .try_send(task)
            .map_err(ClientError::from_string)?;
        Ok(PublicArtifactIdTaskHandle {
            result: PublicArtifactId { id },
            channel: receive_completion_channel,
        })
    }

    #[cfg(feature = "files")]
    fn create_tmp_dir() -> Arc<tempfile::TempDir> {
        Arc::new(
            tempfile::Builder::new()
                .prefix("observation_tools_")
                .tempdir()
                .unwrap(),
        )
    }

    pub async fn shutdown(&self) {
        match &self.params {
            #[cfg(feature = "tokio")]
            TaskLoopParams::TokioRuntime { runtime: _ } => {
                // TODO(doug): Eval runtime
            }
            TaskLoopParams::None => {}
        }
        self.send_task_channel.close();
        let _ = self.receive_shutdown_channel.recv().await;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TaskHandler {
    pub host: String,
    pub token_generator: TokenGenerator,
    pub client: reqwest::Client,
}

impl TaskHandler {
    async fn handle_upload_artifact_task(
        &self,
        task: &UploadArtifactTask,
    ) -> Result<(), GenericError> {
        trace!("Handling artifact: {:?}", task.request);

        let req_b64 = base64::encode(task.request.write_to_bytes()?);
        let mut form = reqwest::multipart::Form::new().text("request", req_b64);
        if let Some(payload) = task.payload.as_ref() {
            let part = match payload {
                #[cfg(feature = "files")]
                UploadArtifactTaskPayload::File(f) => {
                    use tokio_util::codec::BytesCodec;
                    use tokio_util::codec::FramedRead;

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
            })?
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
