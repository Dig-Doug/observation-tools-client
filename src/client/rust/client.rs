use crate::api::new_artifact_id;
use crate::base_artifact_uploader::artifact_group_uploader_data_from_request;
use crate::base_artifact_uploader::BaseArtifactUploaderBuilder;
use crate::base_artifact_uploader::ContextBehavior;
use crate::run_stage_uploader::RunStageUploader;
use crate::run_uploader::RunUploader;
use crate::task_handler::{TaskHandler, TaskLoop};
use crate::upload_artifact_task::UploadArtifactTask;
use crate::upload_artifact_task::UploadArtifactTaskPayload;
use crate::util::{decode_id_proto, time_now};
use crate::util::{ClientError, GenericError};
use crate::PublicArtifactId;
use crate::TokenGenerator;
use crate::UserMetadataBuilder;
use anyhow::anyhow;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_ROOT_GROUP;
use artifacts_api_rust_proto::{public_global_id, StructuredData};
use artifacts_api_rust_proto::{ArtifactGroupUploaderData, ProjectId};
use artifacts_api_rust_proto::{CreateArtifactRequest, PublicGlobalId};

use protobuf::Message;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use std::io::Write;
use std::sync::Arc;

#[cfg(feature = "files")]
use tempfile::NamedTempFile;
#[cfg(feature = "files")]
use tempfile::TempDir;
#[cfg(feature = "tokio")]
use tokio::runtime::Handle;
use tracing::error;
use tracing::trace;
use wasm_bindgen::prelude::*;

pub(crate) const UI_HOST: &str = "https://app.observation.tools";
pub(crate) const API_HOST: &str = "https://api.observation.tools";

#[derive(Clone)]
pub struct ClientOptions {
    pub ui_host: Option<String>,
    pub api_host: Option<String>,
    pub project_id: String,
    pub client: Option<reqwest::Client>,
    pub token_generator: TokenGenerator,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Client {
    pub(crate) options: ClientOptions,
    pub(crate) project_id: ProjectId,
    #[cfg(feature = "files")]
    tmp_dir: Arc<TempDir>,
    task_handler: Arc<TaskHandler>,
    task_loop: Arc<TaskLoop>,
}

pub fn default_reqwest_client() -> reqwest::Client {
    let builder = reqwest::Client::builder().cookie_store(true);
    #[cfg(feature = "cpp")]
    {
        builder = builder.use_rustls_tls();
    }
    builder.build().expect("Failed to build reqwest client")
}

#[cfg(feature = "tokio")]
pub fn create_tokio_runtime() -> Result<Handle, GenericError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    Ok(runtime.handle().clone())
}

#[cfg(feature = "cpp")]
pub(crate) fn ffi_new_client(project_id: String) -> Box<Client> {
    env_logger::init();
    Box::new(Client::new(
        project_id,
        create_tokio_runtime().unwrap(),
        default_reqwest_client(),
    ))
}

#[cfg_attr(feature = "python", pymethods)]
impl Client {
    /*
    // TODO(doug): Figure out why this doesn't work: #[cfg_attr(feature = "python", new)]
    #[cfg(feature = "python")]
    #[new]
    pub fn new(project_id: String) -> Self {
      env_logger::init();
      Self::new_impl(project_id,
                     create_tokio_runtime().unwrap(),
                     default_reqwest_client())
    }
       */

    /*
    pub fn create_run_blocking(&self) -> RunUploader {
        #[cfg(feature = "tokio")]
        self.options.runtime.block_on(self.create_run()).unwrap()
    }
     */
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(
        ui_host: String,
        api_host: String,
        token: String,
        project_id: String,
    ) -> Result<Client, JsValue> {
        Client::new_impl(ClientOptions {
            project_id,
            ui_host: Some(ui_host),
            api_host: Some(api_host),
            token_generator: TokenGenerator::Constant(token),
            client: None,
        })
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub async fn shutdown(self) -> Result<(), ClientError> {
        trace!("Shutting down client");
        self.task_loop.shutdown().await;
        Ok(())
    }

    pub async fn create_run(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<RunUploader, ClientError> {
        let mut request = CreateArtifactRequest::new();
        request.project_id = Some(self.project_id.clone()).into();
        request.artifact_id = Some(new_artifact_id()).into();
        request.run_id.mut_or_insert_default().id = request.artifact_id.clone();
        let group_data = request.mut_artifact_data();
        group_data.user_metadata = Some(metadata.proto.clone()).into();
        group_data.artifact_type = ARTIFACT_TYPE_ROOT_GROUP.into();
        group_data.client_creation_time = Some(time_now()).into();

        self.upload_artifact(&request, None).await.map_err(|e| {
            error!("Failed to create run: {}", e);
            e
        })?;

        Ok(RunUploader {
            base: BaseArtifactUploaderBuilder::default()
                .client(self.clone())
                .data(artifact_group_uploader_data_from_request(&request))
                .context_behavior(ContextBehavior::Init)
                .init(),
        })
    }
}

impl Client {
    /*
    #[cfg(not(feature = "python"))]
    pub fn new(project_id: String, runtime: Handle, client: reqwest::Client) -> Self {
        Self::new_impl(ClientOptions{
            host: env::var("OBS_HOST").unwrap_or("https://api.observation.tools".to_string()),
            project_id,
            runtime,
            client
        })
    }
     */

    pub fn new_impl(options: ClientOptions) -> Result<Self, GenericError> {
        let task_handler = Arc::new(TaskHandler {
            client: options
                .client
                .clone()
                .unwrap_or_else(default_reqwest_client),
            token_generator: options.token_generator.clone(),
            host: options.api_host.clone().unwrap_or(API_HOST.to_string()),
        });

        let task_loop = Arc::new(Self::create_taskloop(task_handler.clone())?);

        let proto: PublicGlobalId = decode_id_proto(&options.project_id)?;
        let project_id = match proto.data {
            Some(public_global_id::Data::ProjectId(project_id)) => project_id,
            _ => Err(anyhow!("Invalid project id: {}", options.project_id))?,
        };

        let client = Client {
            options,
            project_id,
            #[cfg(feature = "files")]
            tmp_dir: Arc::new(
                tempfile::Builder::new()
                    .prefix("observation_tools_")
                    .tempdir()
                    .unwrap(),
            ),
            task_handler,
            task_loop,
        };
        Ok(client)
    }

    #[cfg(feature = "tokio")]
    fn create_taskloop(task_handler: Arc<TaskHandler>) -> Result<TaskLoop, GenericError> {
        let (send_task_channel, receive_task_channel) = async_channel::unbounded();
        let (send_shutdown_channel, receive_shutdown_channel) = async_channel::bounded(1);
        let runtime = create_tokio_runtime()?;
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
    fn create_taskloop(task_handler: Arc<TaskHandler>) -> Result<TaskLoop, GenericError> {
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

    /*
    #[cfg(feature = "cpp")]
    pub(crate) fn ffi_create_run(&self) -> Box<RunUploader> {
      let uploader = self.runtime.block_on(self.create_run()).unwrap();
      Box::new(uploader)
    }
     */

    fn deserialize_run_stage(&self, serialized: String) -> RunStageUploader {
        RunStageUploader {
            base: BaseArtifactUploaderBuilder::default()
                .client(self.clone())
                .data(
                    ArtifactGroupUploaderData::parse_from_bytes(
                        &base64::decode(serialized).unwrap(),
                    )
                    .unwrap(),
                )
                .context_behavior(ContextBehavior::Init)
                .init(),
        }
    }

    #[cfg(feature = "cpp")]
    pub(crate) fn ffi_deserialize_run_stage(&self, serialized: String) -> Box<RunStageUploader> {
        Box::new(self.deserialize_run_stage(serialized))
    }

    pub(crate) async fn upload_artifact(
        &self,
        request: &CreateArtifactRequest,
        structured_data: Option<StructuredData>,
    ) -> Result<PublicArtifactId, ClientError> {
        let bytes = structured_data.and_then(|s| s.write_to_bytes().ok());
        self.upload_artifact_raw_bytes(request, bytes.as_ref().map(|b| b.as_slice()))
            .await
    }

    pub(crate) async fn upload_artifact_raw_bytes(
        &self,
        request: &CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> Result<PublicArtifactId, ClientError> {
        let task = self.new_task(request, raw_data);
        trace!("Enqueueing task: {:#?}", task);
        self.task_handler.handle_upload_artifact_task(&task).await
    }

    #[cfg(feature = "files")]
    fn new_task(
        &self,
        request: &CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> UploadArtifactTask {
        let tmp_file = if let Some(raw_data_slice) = raw_data {
            // TODO(doug): Consider using a spooled tempfile
            let mut tmp_file = NamedTempFile::new_in(&*self.tmp_dir).unwrap();
            tmp_file.write_all(raw_data_slice).unwrap();
            Some(tmp_file)
        } else {
            None
        };

        UploadArtifactTask {
            request: request.clone(),
            payload: tmp_file.map(|f| UploadArtifactTaskPayload::File(f)),
        }
    }

    #[cfg(not(feature = "files"))]
    fn new_task(
        &self,
        request: &CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> UploadArtifactTask {
        UploadArtifactTask {
            request: request.clone(),
            // TODO(doug): Do we need to make a copy of the raw data here?
            payload: raw_data.map(|bytes| UploadArtifactTaskPayload::Bytes(bytes.to_vec())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::SphereBuilder;
    use crate::client::default_reqwest_client;
    use crate::util::new_uuid_proto;
    use crate::util::time_now;
    use crate::util::GenericError;
    use crate::Client;
    use artifacts_api_rust_proto::ArtifactUserMetadata;
    use artifacts_api_rust_proto::CreateArtifactRequest;
    #[cfg(feature = "bazel")]
    use runfiles::Runfiles;
    use std::env;
    use std::path::Path;
    use std::path::PathBuf;
    use tokio::runtime::Handle;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[cfg(feature = "bazel")]
    fn get_test_data_path(filename: &str) -> String {
        let r = Runfiles::create().unwrap();
        r.rlocation(format!(
            "observation_tools_client/src/client/rust/{}",
            filename
        ))
        .into_os_string()
        .into_string()
        .unwrap()
    }

    #[cfg(not(feature = "bazel"))]
    fn get_test_data_path(filename: &str) -> String {
        filename.to_string()
    }

    fn get_test_output_dir() -> PathBuf {
        env::var("TEST_UNDECLARED_OUTPUTS_DIR")
            .map(|p| Path::new(&p).to_path_buf())
            .unwrap_or(env::current_dir().expect("Failed to get working dir"))
    }

    fn project_id() -> String {
        env::var("OBS_PROJECT_ID").unwrap()
    }

    fn create_client() -> Client {
        Client::new(project_id(), Handle::current(), default_reqwest_client())
    }

    #[tokio::test]
    async fn upload_shared_artifact() -> Result<(), GenericError> {
        init();
        let client = create_client();

        let sphere = SphereBuilder::new(64.0);

        let project_id = project_id();
        let metadata = ArtifactUserMetadata::new();

        let mut request = CreateArtifactRequest::new();
        request.project_id = Some(project_id.clone()).into();
        //request.run_id = Some(parent_data.get_run_id().clone()).into();
        request.mut_artifact_id().uuid = Some(new_uuid_proto()).into();
        let artifact_data = request.mut_artifact_data();
        artifact_data.user_metadata = Some(metadata).into();
        artifact_data.client_creation_time = Some(time_now()).into();

        let source_data_id = client.upload_artifact(&request, Some((&sphere).into()));

        client.shutdown().await?;
        Ok(())
    }
}
