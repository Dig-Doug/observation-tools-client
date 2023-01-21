use crate::base_artifact_uploader::{artifact_group_uploader_data_from_request, ContextBehavior};
use wasm_bindgen::prelude::*;
use crate::base_artifact_uploader::{BaseArtifactUploaderBuilder};
use serde::{Deserialize, Serialize};
use artifacts_api_rust_proto::{ArtifactGroupUploaderData, CreateArtifactRequest, StructuredData};
use protobuf::{Message};
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(not(feature = "wasm"))]
use std::env;
use std::io::Write;
use std::sync::Arc;
use async_channel::{Receiver, Sender};
use futures::TryFutureExt;
#[cfg(feature = "files")]
use tempfile::{NamedTempFile, TempDir};
#[cfg(feature = "tokio")]
use tokio::{
  runtime::{Handle},
};
use tracing::{error, trace};
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_ROOT_GROUP;
use crate::api::new_artifact_id;
use crate::run_stage_uploader::RunStageUploader;
use crate::run_uploader::RunUploader;
use crate::task_handler::TaskHandler;
use crate::TokenGenerator;
use crate::upload_artifact_task::{UploadArtifactTask, UploadArtifactTaskPayload};
use crate::util::{ClientError, GenericError, time_now};


#[derive(Clone)]
pub struct ClientOptions {
  pub host: String,
  pub project_id: String,
  #[cfg(feature = "tokio")]
  pub runtime: Handle,
  pub client: reqwest::Client,
  pub token_generator: TokenGenerator,
}

#[wasm_bindgen]
#[derive(Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Client {
  pub(crate) options: ClientOptions,
  #[cfg(feature = "files")]
  tmp_dir: Arc<TempDir>,
  task_handler: Arc<TaskHandler>,
  send_task_channel: Sender<UploadArtifactTask>,
  receive_shutdown_channel: Receiver<()>,
}

#[cfg(feature = "cpp")]
fn default_reqwest_client() -> reqwest::Client {
  reqwest::Client::builder()
    .use_rustls_tls()
    .build().expect("Failed to build reqwest client")
}

#[cfg(feature = "tokio")]
fn create_tokio_runtime() -> Result<Handle, GenericError> {
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;
  Ok(runtime.handle().clone())
}

#[cfg(feature = "cpp")]
pub(crate) fn ffi_new_client(project_id: String) -> Box<Client> {
  env_logger::init();
  Box::new(Client::new(project_id, create_tokio_runtime().unwrap(), default_reqwest_client()))
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
    api_host: String,
    token: String,
    project_id: String,
  ) -> Self {
    Client::new_impl(ClientOptions {
      project_id,
      host: api_host,
      token_generator: TokenGenerator::Constant(token),
      client: reqwest::Client::new(),
    })
  }

  pub async fn shutdown(self) -> Result<(), ClientError> {
    trace!("Shutting down client");
    self.send_task_channel.close();
    let _ = self.receive_shutdown_channel.recv().await;
    Ok(())
  }

  pub async fn create_run(&self) -> Result<(), ClientError> {
    let mut request = CreateArtifactRequest::new();
    request.project_id = self.options.project_id.clone();
    request.artifact_id = Some(new_artifact_id()).into();
    //request.run_id = request.artifact_id.clone();
    let group_data = request.mut_artifact_data();
    //group_data.user_metadata = Some(metadata).into();
    group_data.artifact_type = ARTIFACT_TYPE_ROOT_GROUP.into();
    group_data.client_creation_time = Some(time_now()).into();

    self.upload_artifact(&request, None).await;

    let todo = RunUploader {
      base: BaseArtifactUploaderBuilder::default()
        .client(self.clone())
        .data(artifact_group_uploader_data_from_request(&request))
        .context_behavior(ContextBehavior::Init)
        .init(),
    };

    Ok(())
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

  pub fn new_impl(options: ClientOptions) -> Self {
    let (send_task_channel, receive_task_channel) = async_channel::unbounded();
    let (send_shutdown_channel, receive_shutdown_channel) = async_channel::bounded(1);

    let task_handler = Arc::new(TaskHandler {
      client: options.client.clone(),
      token_generator: options.token_generator.clone(),
      host: options.host.clone(),
      receive_task_channel,
      send_shutdown_channel,
    });
    #[cfg(feature = "tokio")]
    options.runtime.spawn(task_handler.run());
    let client = Client {
      options,
      #[cfg(feature = "files")]
      tmp_dir: Arc::new(
        tempfile::Builder::new()
          .prefix("observation_tools_")
          .tempdir()
          .unwrap(),
      ),
      task_handler,
      send_task_channel,
      receive_shutdown_channel,
    };
    client
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
        .data(ArtifactGroupUploaderData::parse_from_bytes(&base64::decode(serialized).unwrap()).unwrap())
        .context_behavior(ContextBehavior::Init)
        .init(),
    }
  }

  #[cfg(feature = "cpp")]
  pub(crate) fn ffi_deserialize_run_stage(&self, serialized: String) -> Box<RunStageUploader> {
    Box::new(self.deserialize_run_stage(serialized))
  }

  pub(crate) async fn upload_artifact(&self, request: &CreateArtifactRequest,
                                      structured_data: Option<StructuredData>) {
    let bytes = structured_data.and_then(|s| s.write_to_bytes().ok());
    self.upload_artifact_raw_bytes(request, bytes.as_ref().map(|b| b.as_slice())).await
  }

  pub(crate) async fn upload_artifact_raw_bytes(&self, request: &CreateArtifactRequest, raw_data: Option<&[u8]>) {
    let task = self.new_task(request, raw_data);
    trace!("Enqueueing task: {:?}", task);
    let result = self.task_handler.handle_upload_artifact_task(&task).await;
    if let Err(e) = result {
      error!("Failed to upload artifact: {}", e);
    }
    /*
    let result = self.send_task_channel.send_blocking(task);
    if let Err(e) = result {
      error!("Failed to enqueue task: {}", e.to_string());
    }
     */
  }

  #[cfg(feature = "files")]
  fn new_task(&self, request: &CreateArtifactRequest, raw_data: Option<&[u8]>) -> UploadArtifactTask {
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
  fn new_task(&self, request: &CreateArtifactRequest, raw_data: Option<&[u8]>) -> UploadArtifactTask {
    UploadArtifactTask {
      request: request.clone(),
      // TODO(doug): Do we need to make a copy of the raw data here?
      payload: raw_data.map(|bytes| UploadArtifactTaskPayload::Bytes(bytes.to_vec())),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::env;
  use std::path::{Path, PathBuf};
  use tracing::trace;
  use tokio::runtime::Handle;
  use artifacts_api_rust_proto::{ArtifactUserMetadata, CreateArtifactRequest};
  use crate::Client;
  #[cfg(feature = "bazel")]
  use runfiles::Runfiles;
  use crate::api::SphereBuilder;
  use crate::client::default_reqwest_client;
  use crate::util::{GenericError, new_uuid_proto, time_now};

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[cfg(feature = "bazel")]
  fn get_test_data_path(filename: &str) -> String {
    let r = Runfiles::create().unwrap();
    r.rlocation(format!("observation_tools_client/src/client/rust/{}", filename))
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
