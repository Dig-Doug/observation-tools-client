use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::RunUploader;
use crate::task_handle::TaskHandle;
use crate::throttle_without_access_cookie::ThrottleWithoutAccessCookieLayer;
use crate::upload_artifact::UploadArtifactService;
use crate::upload_artifact::UploadArtifactTask;
use crate::upload_artifact::UploadArtifactTaskPayload;
use crate::util::time_now;
use crate::util::ClientError;
use crate::util::GenericError;
use crate::PublicArtifactId;
use crate::PublicArtifactIdTaskHandle;
use crate::TokenGenerator;
use anyhow::anyhow;
use core::fmt::Debug;
use core::fmt::Formatter;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifact::ArtifactType;
use observation_tools_common::artifact::StructuredData;
use observation_tools_common::artifacts::UserMetadata;
use observation_tools_common::create_artifact::CreateArtifactRequest;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use observation_tools_common::GlobalId;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tower::buffer::Buffer;
use tower::util::BoxService;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_service::Service;
use tracing::debug;
use tracing::error;
use tracing::warn;
use url::Url;
use wasm_bindgen::prelude::*;

pub(crate) const UI_HOST: &str = "https://app.observation.tools";
pub(crate) const API_HOST: &str = "https://api.observation.tools";

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub ui_host: Option<String>,
    pub api_host: Option<String>,
    pub reqwest_client: Option<reqwest::Client>,
    pub token_generator: TokenGenerator,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            ui_host: None,
            api_host: None,
            reqwest_client: None,
            #[cfg(feature = "tokio")]
            token_generator: TokenGenerator::OAuth2BrowserFlow,
            #[cfg(not(feature = "tokio"))]
            token_generator: TokenGenerator::OAuth2DeviceCodeFlow,
        }
    }
}

#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Clone, Debug)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

pub struct ClientInner {
    pub(crate) options: ClientOptions,
    pub(crate) project_id: ProjectId,
    task_handler: Buffer<BoxService<UploadArtifactTask, (), GenericError>, UploadArtifactTask>,
    params: TaskRuntimeParams,
    #[cfg(feature = "files")]
    tmp_dir: tempfile::TempDir,
}

#[derive(Debug)]
pub enum TaskRuntimeParams {
    #[cfg(feature = "tokio")]
    TokioRuntime {
        handle: tokio::runtime::Handle,
        /// If we create our own runtime, hold a reference to prevent it from
        /// being dropped
        runtime: Option<Arc<tokio::runtime::Runtime>>,
        // When TaskTracker is not set, we block on the task
        // TODO(doug): I believe that python is blocking the rust threads from running the
        // background, but I haven't confirmed. In theory we should not need to block in python.
        task_tracker: tokio_util::task::TaskTracker,
    },
    #[cfg(not(feature = "tokio"))]
    WasmRuntime {
        // TODO(doug): Is there a library we can use to keep track of tasks like in tokio
        send_task_completion: async_channel::Sender<()>,
        receive_task_completion: async_channel::Receiver<()>,
    },
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Client {
    #[new]
    fn py_new(project_id: String, api_host: Option<String>) -> pyo3::PyResult<Self> {
        Ok(Client::new(
            project_id,
            ClientOptions {
                api_host,
                ..Default::default()
            },
        )?)
    }

    #[pyo3(name = "create_run")]
    pub fn create_run_py(&self, metadata: &UserMetadata) -> Result<RunUploader, ClientError> {
        self.create_run(metadata.clone())
    }

    #[pyo3(name = "shutdown")]
    pub fn shutdown_py(&mut self) {
        self.inner.shutdown_sync()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Client {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_wasm(
        ui_host: Option<String>,
        api_host: Option<String>,
        project_id: String,
    ) -> Result<Client, JsValue> {
        Client::new(
            project_id,
            ClientOptions {
                ui_host,
                api_host,
                token_generator: TokenGenerator::OAuth2DeviceCodeFlow,
                reqwest_client: None,
            },
        )
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub async fn shutdown(mut self) -> Result<(), ClientError> {
        self.inner.shutdown_async().await
    }

    pub fn create_run_js(&self, metadata: &UserMetadata) -> Result<RunUploader, ClientError> {
        self.create_run(metadata.clone())
    }
}

impl Client {
    pub fn new(project_id: String, options: ClientOptions) -> Result<Self, ClientError> {
        Ok(Self {
            inner: Arc::new(ClientInner::new(project_id, options)?),
        })
    }

    pub(crate) fn upload_artifact(
        &self,
        request: CreateArtifactRequest,
        structured_data: Option<StructuredData>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let bytes = structured_data
            .map(|s| rmp_serde::to_vec(&s))
            .transpose()
            .map_err(ClientError::from_string)?;
        self.upload_artifact_raw_bytes(request, bytes.as_ref().map(|b| b.as_slice()))
    }

    pub(crate) fn upload_artifact_raw_bytes(
        &self,
        request: CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.inner.submit_task(request, raw_data)
    }

    pub fn create_run<M: Into<UserMetadata>>(
        &self,
        into_metadata: M,
    ) -> Result<RunUploader, ClientError> {
        let artifact_id = ArtifactId::new();
        let request = CreateArtifactRequest {
            project_id: self.inner.project_id.clone(),
            artifact_id: artifact_id.clone(),
            run_id: Some(RunId { id: artifact_id }),
            payload: ArtifactData {
                user_metadata: into_metadata.into(),
                artifact_type: ArtifactType::RootGroup,
                client_creation_time: time_now(),
                ancestor_group_ids: vec![],
            },
            series_point: None,
        };
        let handle = self.upload_artifact(request, None)?;
        Ok(RunUploader {
            base: BaseArtifactUploader {
                client: self.clone(),
                project_id: self.inner.project_id.clone(),
                run_id: RunId {
                    id: handle.id.clone(),
                },
                id: handle.id.clone(),
                ancestor_group_ids: vec![],
                handle,
            },
        })
    }
}

impl ClientInner {
    pub fn new(project_id: String, options: ClientOptions) -> Result<Self, ClientError> {
        debug!("Creating client");

        let api_host = options.api_host.clone().unwrap_or(API_HOST.to_string());
        //let cookie_store = Arc::new(cookie::Jar::default());
        let service = ServiceBuilder::new()
            .layer(ThrottleWithoutAccessCookieLayer {
                //cookie_store: cookie_store.clone(),
                cookie_store: Arc::new(()),
                api_host: Url::parse(&api_host).map_err(ClientError::from_string)?,
            })
            .service(UploadArtifactService {
                client: options.reqwest_client.clone().unwrap_or_else(|| {
                    let builder = reqwest::Client::builder();
                    //.cookie_provider(cookie_store.clone());
                    builder.build().expect("Failed to build reqwest client")
                }),
                token_generator: options.token_generator.clone(),
                host: api_host,
            })
            .boxed();

        let (task_buffer, worker) = Buffer::pair(service, 100);

        #[cfg(feature = "tokio")]
        let params = Self::get_or_create_tokio_runtime()?;
        #[cfg(not(feature = "tokio"))]
        let params = {
            debug!("Using wasm runtime");
            let (send_task_completion, receive_task_completion) = async_channel::unbounded();
            TaskRuntimeParams::WasmRuntime {
                send_task_completion,
                receive_task_completion,
            }
        };

        match &params {
            #[cfg(feature = "tokio")]
            TaskRuntimeParams::TokioRuntime { handle, .. } => {
                handle.spawn(worker);
            }
            #[cfg(not(feature = "tokio"))]
            TaskRuntimeParams::WasmRuntime { .. } => {
                wasm_bindgen_futures::spawn_local(worker);
            }
        }

        let global_id: GlobalId = project_id
            .clone()
            .try_into()
            .map_err(ClientError::from_string)?;
        let project_id = match global_id {
            GlobalId::Project(project_id) => project_id,
            _ => Err(anyhow!(
                "The id was valid but not a project id: {}",
                project_id
            ))?,
        };

        let client = ClientInner {
            options,
            project_id,
            task_handler: task_buffer,
            params,
            #[cfg(feature = "files")]
            tmp_dir: Self::create_tmp_dir(),
        };
        Ok(client)
    }

    fn submit_task(
        &self,
        request: CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let id = request.artifact_id.clone();

        #[cfg(feature = "files")]
        let payload = {
            let tmp_file = if let Some(raw_data_slice) = raw_data {
                // TODO(doug): Consider using a spooled tempfile
                let mut tmp_file = tempfile::NamedTempFile::new_in(&self.tmp_dir).unwrap();
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
            #[cfg(not(feature = "tokio"))]
            shared_completion_channel: match &self.params {
                TaskRuntimeParams::WasmRuntime {
                    send_task_completion,
                    ..
                } => send_task_completion.clone(),
                _ => {
                    panic!("Shared completion channel not supported in tokio")
                }
            },
        };
        debug!(
            "Submitting artifact: {} raw_data_len: {:?}",
            task.artifact_global_id(),
            raw_data.map(|d| d.len() as i64)
        );

        let mut task_handler = self.task_handler.clone();
        let upload_task = async move {
            debug!("Upload task for {} started", task.artifact_global_id());
            match task_handler.ready().await {
                Ok(service) => {
                    debug!("Upload task for {} ready", task.artifact_global_id());
                    if let Err(e) = service.call(task).await {
                        error!("Failed to upload artifact: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to upload artifact: {}", e);
                }
            }
        };

        match &self.params {
            #[cfg(feature = "tokio")]
            TaskRuntimeParams::TokioRuntime {
                handle,
                task_tracker,
                ..
            } => {
                debug!("Spawning task on tokio runtime");
                task_tracker.spawn_on(upload_task, handle);
            }
            #[cfg(not(feature = "tokio"))]
            TaskRuntimeParams::WasmRuntime { .. } => {
                wasm_bindgen_futures::spawn_local(upload_task);
            }
        }

        Ok(PublicArtifactIdTaskHandle {
            result: PublicArtifactId { id },
            channel: receive_completion_channel,
        })
    }

    #[cfg(feature = "files")]
    fn create_tmp_dir() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("observation_tools_")
            .tempdir()
            .unwrap()
    }

    #[cfg(feature = "tokio")]
    fn get_or_create_tokio_runtime() -> Result<TaskRuntimeParams, ClientError> {
        let (handle, runtime) = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                debug!("Using existing tokio runtime");
                (handle, None)
            }
            Err(_e) => {
                debug!("No tokio runtime found, spawning one");
                let runtime = Arc::new(
                    tokio::runtime::Builder::new_multi_thread()
                        // TODO(doug): Figure out the best number of threads
                        .worker_threads(4)
                        .enable_all()
                        .build()?,
                );
                (runtime.handle().clone(), Some(runtime))
            }
        };
        Ok(TaskRuntimeParams::TokioRuntime {
            handle,
            runtime,
            task_tracker: tokio_util::task::TaskTracker::new(),
        })
    }

    fn shutdown_sync(&self) {
        match &self.params {
            #[cfg(feature = "tokio")]
            TaskRuntimeParams::TokioRuntime {
                handle,
                task_tracker,
                ..
            } => {
                debug!("Closing task tracker");
                task_tracker.close();

                let (send_tasks_done, receive_tasks_done) = std::sync::mpsc::channel();
                let task_tracker = task_tracker.clone();
                debug!("Spawning a task to wait for other tasks to complete");
                handle.spawn(async move {
                    warn!("Waiting for tasks to complete");
                    task_tracker.wait().await;
                    if let Err(e) = send_tasks_done.send(()) {
                        error!("Failed to send tasks done signal: {}", e);
                    }
                });
                debug!("Waiting for task completion signal");
                if let Err(e) = receive_tasks_done.recv_timeout(Duration::from_secs(60)) {
                    error!("Failed to wait for tasks to complete: {}", e);
                }
            }
            #[cfg(not(feature = "tokio"))]
            TaskRuntimeParams::WasmRuntime { .. } => {
                // Do nothing, we can't do a blocking wait
                panic!("shutdown_sync() is not supported in the browser")
            }
        }
    }

    async fn shutdown_async(&mut self) -> Result<(), ClientError> {
        debug!("Shutting down client");
        match &self.params {
            #[cfg(feature = "tokio")]
            TaskRuntimeParams::TokioRuntime { .. } => {
                panic!("shutdown() is not supported in the browser")
            }
            #[cfg(not(feature = "tokio"))]
            TaskRuntimeParams::WasmRuntime {
                send_task_completion,
                receive_task_completion,
            } => {
                while send_task_completion.sender_count() > 1 {
                    debug!("Waiting for all tasks to complete...");
                    // Do nothing, wait until all tasks are done
                    let _unused = receive_task_completion.recv().await;
                }
            }
        }
        debug!("Finished shutting down client");
        Ok(())
    }
}

impl Debug for ClientInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Client")
            .field("project_id", &self.project_id)
            .field("options", &self.options)
            .finish()
    }
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        self.shutdown_sync();
    }
}
