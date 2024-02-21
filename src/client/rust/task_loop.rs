use crate::generated::CreateArtifactRequest;
use crate::task_handle::TaskHandle;
use crate::token_generator::TokenGenerator;
use crate::util::ClientError;
use crate::util::GenericError;
use crate::PublicArtifactId;
use crate::PublicArtifactIdTaskHandle;
use base64::Engine;
use futures::future::BoxFuture;
use futures::ready;
use pin_project::pin_project;
use protobuf::Message;
use reqwest::cookie;
use reqwest::multipart::Part;
use std::future::Future;
#[cfg(feature = "files")]
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Condvar;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::Semaphore;
use tokio_util::sync::PollSemaphore;
use tokio_util::task::TaskTracker;
use tower::limit::concurrency::future::ResponseFuture;
use tower::limit::ConcurrencyLimit;
use tower::make::Shared;
use tower::util::BoxCloneService;
use tower::util::BoxService;
use tower::Service;
use tower::ServiceExt;
use tracing::error;
use tracing::trace;
use url::Url;

#[derive(Debug)]
pub struct TaskLoop {
    task_handler: BoxCloneService<UploadArtifactTask, (), GenericError>,
    params: TaskLoopParams,
    #[cfg(feature = "files")]
    tmp_dir: Arc<tempfile::TempDir>,
}

#[derive(Debug)]
pub enum TaskLoopParams {
    #[cfg(feature = "tokio")]
    TokioRuntime {
        handle: tokio::runtime::Handle,
        /// If we create our own runtime, hold a reference to prevent it from
        /// being dropped
        runtime: Option<Arc<tokio::runtime::Runtime>>,
        task_tracker: TaskTracker,
    },
    #[cfg(not(feature = "tokio"))]
    None {
        send_task_channel: async_channel::Sender<UploadArtifactTask>,
        receive_shutdown_channel: async_channel::Receiver<()>,
    },
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
    #[cfg(not(feature = "files"))]
    Bytes(Vec<u8>),
}

impl TaskLoop {
    pub(crate) fn new(
        task_handler: BoxCloneService<UploadArtifactTask, (), GenericError>,
    ) -> Result<TaskLoop, GenericError> {
        #[cfg(feature = "tokio")]
        let params = {
            let (handle, runtime) = match tokio::runtime::Handle::try_current() {
                Ok(handle) => (handle, None),
                Err(_e) => {
                    let runtime = Arc::new(
                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()?,
                    );
                    (runtime.handle().clone(), Some(runtime))
                }
            };
            TaskLoopParams::TokioRuntime {
                handle,
                runtime,
                task_tracker: TaskTracker::new(),
            }
        };
        #[cfg(not(feature = "tokio"))]
        let params = {
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
            wasm_bindgen_futures::spawn_local(task_loop);
            TaskLoopParams::None {
                receive_shutdown_channel,
                send_task_channel,
            }
        };

        Ok(TaskLoop {
            task_handler,
            params,
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

        match &self.params {
            #[cfg(feature = "tokio")]
            TaskLoopParams::TokioRuntime {
                handle,
                task_tracker,
                ..
            } => {
                let mut task_handler = self.task_handler.clone();
                task_tracker.spawn_on(
                    async move {
                        trace!("Waiting for service ready");
                        match task_handler.ready().await {
                            Ok(service) => {
                                trace!("Service ready");
                                if let Err(e) = service.call(task).await {
                                    error!("Failed to upload artifact: {}", e);
                                }
                                trace!("Service done");
                            }
                            Err(e) => {
                                error!("Failed to upload artifact: {}", e);
                            }
                        }
                    },
                    handle,
                );
            }
            #[cfg(not(feature = "tokio"))]
            TaskLoopParams::None => {
                self.send_task_channel
                    .try_send(task)
                    .map_err(ClientError::from_string)?;
            }
        }

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
            TaskLoopParams::TokioRuntime { .. } => {
                // All clean up is done in Drop
                error!("shutdown() is not required for rust");
            }
            #[cfg(not(feature = "tokio"))]
            TaskLoopParams::None {
                send_task_channel,
                receive_shutdown_channel,
            } => {
                send_task_channel.close();
                let _ = receive_shutdown_channel.recv().await;
            }
        }
    }
}

impl Drop for TaskLoop {
    fn drop(&mut self) {
        match &self.params {
            #[cfg(feature = "tokio")]
            TaskLoopParams::TokioRuntime {
                handle,
                task_tracker,
                ..
            } => {
                task_tracker.close();

                let (send_tasks_done, receive_tasks_done) = std::sync::mpsc::channel();
                let task_tracker = task_tracker.clone();
                handle.spawn(async move {
                    error!("Waiting for tasks to complete");
                    task_tracker.wait().await;
                    if let Err(e) = send_tasks_done.send(()) {
                        error!("Failed to send tasks done signal: {}", e);
                    }
                });
                if let Err(e) = receive_tasks_done.recv_timeout(Duration::from_secs(60)) {
                    error!("Failed to wait for tasks to complete: {}", e);
                }
            }
            #[cfg(not(feature = "tokio"))]
            TaskLoopParams::None => {
                warn!("Users must call shutdown()");
            }
        }
    }
}
