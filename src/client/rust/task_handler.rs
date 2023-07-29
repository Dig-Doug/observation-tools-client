use crate::token_generator::TokenGenerator;
use crate::upload_artifact_task::UploadArtifactTask;
use crate::upload_artifact_task::UploadArtifactTaskPayload;
use crate::util::ClientError;
use crate::util::GenericError;
use crate::PublicArtifactId;
use futures::AsyncWriteExt;
use protobuf::Message;
use reqwest::multipart::Part;
use std::sync::Arc;
#[cfg(feature = "tokio")]
use tokio_util::codec::BytesCodec;
#[cfg(feature = "tokio")]
use tokio_util::codec::FramedRead;
use tracing::error;
use tracing::trace;
use wasm_bindgen::closure::Closure;

pub enum TaskLoop {
    #[cfg(feature = "tokio")]
    TokioRuntime {
        runtime: tokio::runtime::Handle,
        send_task_channel: async_channel::Sender<UploadArtifactTask>,
        receive_shutdown_channel: async_channel::Receiver<()>,
    },
    WindowEventLoop {
        send_task_channel: crossbeam_channel::Sender<UploadArtifactTask>,
        receive_shutdown_channel: crossbeam_channel::Receiver<()>,
        closure: Closure<dyn FnMut()>,
        interval_id: i32,
    },
}

impl TaskLoop {
    pub async fn shutdown(&self) {
        match self {
            #[cfg(feature = "tokio")]
            TaskLoop::TokioRuntime {
                send_task_channel,
                receive_shutdown_channel,
                ..
            } => {
                send_task_channel.close();
                let _ = receive_shutdown_channel.recv().await;
                // TODO(doug): Eval runtime
            }
            TaskLoop::WindowEventLoop { .. } => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct TaskHandler {
    pub host: String,
    pub token_generator: TokenGenerator,
    pub client: reqwest::Client,
}

impl TaskHandler {
    #[cfg(feature = "tokio")]
    pub(crate) fn create_taskloop(
        task_handler: Arc<TaskHandler>,
    ) -> Result<TaskLoop, GenericError> {
        let (send_task_channel, receive_task_channel) = async_channel::unbounded();
        let (send_shutdown_channel, receive_shutdown_channel) = async_channel::bounded(1);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let runtime = runtime.handle().clone();
        runtime.spawn(async move {
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

            if let Err(e) = send_shutdown_channel.send(()).await {
                error!("Error shutting down: {}", e);
            }
        });

        Ok(TaskLoop::TokioRuntime {
            send_task_channel,
            receive_shutdown_channel,
            runtime,
        })
    }

    #[cfg(not(feature = "tokio"))]
    pub(crate) fn create_taskloop(
        task_handler: Arc<TaskHandler>,
    ) -> Result<TaskLoop, GenericError> {
        let (send_task_channel, receive_task_channel) = crossbeam_channel::unbounded();
        let (send_shutdown_channel, receive_shutdown_channel) = crossbeam_channel::bounded(1);
        let run_closure = {
            let receive_task_channel = receive_task_channel.clone();
            Closure::new(move || {
                while !receive_task_channel.is_closed() || !receive_task_channel.is_empty() {
                    let task = receive_task_channel.try_recv();
                    match task {
                        Ok(t) => {
                            wasm_bindgen_futures::spawn_local(async move {
                                let result = task_handler.handle_upload_artifact_task(&t).await;
                                if let Err(e) = result {
                                    error!("Failed to upload artifact: {}", e);
                                }
                            });
                        }
                        Err(_) => {
                            error!("Failed task");
                            panic!("Failed to receive task");
                        }
                    }
                }
            })
        };
        let window = web_sys::window().unwrap();
        let run_interval_id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                run_closure.as_ref().unchecked_ref(),
                Duration::from_millis(10).as_millis() as i32,
            )
            .expect("Should register `setTimeout`.");

        Ok(TaskLoop::WindowEventLoop {
            closure: run_closure,
            interval_id: run_interval_id,
            receive_shutdown_channel,
            send_task_channel,
        })
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
            Ok(PublicArtifactId {
                id: task
                    .request
                    .artifact_id
                    .as_ref()
                    .expect("ArtifactId missing on upload task")
                    .clone(),
            })
        }
    }
}

impl Drop for TaskHandler {
    fn drop(&mut self) {
        trace!("Task handler dropped");
    }
}
