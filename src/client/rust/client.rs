use std::cell::RefCell;
use crate::base_artifact_uploader::ContextBehavior;
use crate::base_artifact_uploader::{BaseArtifactUploaderBuilder};
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
  task::JoinHandle,
};
#[cfg(feature = "tokio")]
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::task_handler::TaskHandler;
use crate::upload_artifact_task::{UploadArtifactTask, UploadArtifactTaskPayload};
use crate::util::{ClientError, encode_id_proto, GenericError, new_uuid_proto, time_now};


#[derive(Clone)]
pub enum TokenGenerator {
  Empty,
}

impl TokenGenerator {
  pub async fn token(&self) -> Result<String, std::io::Error> {
    Ok("test".to_string())
  }
}

#[derive(Clone)]
pub struct ClientOptions {
  pub host: String,
  pub project_id: String,
  #[cfg(feature = "tokio")]
  pub runtime: Handle,
  pub client: reqwest::Client,
}

#[derive(Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Client {
  options: ClientOptions,
  tmp_dir: Arc<TempDir>,
  token_generator: TokenGenerator,
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

  fn new_impl(options: ClientOptions) -> Self {
    let token_generator = TokenGenerator::Empty;
    let (send_task_channel, receive_task_channel) = async_channel::unbounded();
    let (send_shutdown_channel, receive_shutdown_channel) = async_channel::bounded(1);

    let task_handler = TaskHandler {
      client: options.client.clone(),
      token_generator: token_generator.clone(),
      host: options.host.clone(),
      receive_task_channel,
      send_shutdown_channel,
    };
    #[cfg(feature = "tokio")]
    options.runtime.spawn(task_handler.run());
    let client = Client {
      options,
      tmp_dir: Arc::new(
        tempfile::Builder::new()
          .prefix("observation_tools_")
          .tempdir()
          .unwrap(),
      ),
      token_generator,
      send_task_channel,
      receive_shutdown_channel,
    };
    client
  }

  pub async fn shutdown(self) -> Result<(), GenericError> {
    self.send_task_channel.close();
    let _ = self.receive_shutdown_channel.recv().await;
    Ok(())
  }

  pub async fn create_run(&self) -> Result<RunUploader, GenericError> {
    let response = self.create_run_request().await?.send().await?;
    self.parse_create_run_response(response).await
  }

  pub async fn create_run_request(&self) -> Result<RequestBuilder, GenericError> {
    let mut request = CreateRunRequest::new();
    request.set_project_id(self.options.project_id.clone());
    request.mut_run_data().set_client_creation_time(time_now());

    let mut params = HashMap::new();
    params.insert("request", base64::encode(request.write_to_bytes().unwrap()));
    let token = self.token_generator.token().await?;
    let request_builder = self
      .options
      .client
      .post(format!("{}/create-run", self.options.host))
      .bearer_auth(token)
      .form(&params);
    Ok(request_builder)
  }

  pub async fn parse_create_run_response(&self, response: Response) -> Result<RunUploader, GenericError> {
    if response.status().is_server_error() {
      debug!("{:?}", response);
      panic!("Server error {:?}", response)
    }

    let response_body = response.text().await?;
    let response: CreateRunResponse =
      parse_from_bytes(&decode(response_body).unwrap()).unwrap();
    let mut new_data = ArtifactGroupUploaderData::new();
    new_data.set_project_id(self.options.project_id.clone());
    new_data.set_run_id(response.get_run_id().clone());
    new_data.set_id(response.get_root_stage_id().clone());
    Ok(RunUploader {
      base: BaseArtifactUploaderBuilder::default()
        .client(self.clone())
        .data(new_data)
        .context_behavior(ContextBehavior::Init)
        .init(),
      response,
    })
  }

  #[cfg(feature = "cpp")]
  pub(crate) fn ffi_create_run(&self) -> Box<RunUploader> {
    let uploader = self.runtime.block_on(self.create_run()).unwrap();
    Box::new(uploader)
  }

  fn deserialize_run_stage(&self, serialized: String) -> RunStageUploader {
    RunStageUploader {
      base: BaseArtifactUploaderBuilder::default()
        .client(self.clone())
        .data(parse_from_bytes(&base64::decode(serialized).unwrap()).unwrap())
        .context_behavior(ContextBehavior::Init)
        .init(),
    }
  }

  #[cfg(feature = "cpp")]
  pub(crate) fn ffi_deserialize_run_stage(&self, serialized: String) -> Box<RunStageUploader> {
    Box::new(self.deserialize_run_stage(serialized))
  }

  pub(crate) fn upload_artifact(&self, request: &CreateArtifactRequest,
                                structured_data: Option<StructuredData>) {
    let bytes = structured_data.and_then(|s| s.write_to_bytes().ok());
    self.upload_artifact_raw_bytes(request, bytes.as_ref().map(|b| b.as_slice()))
  }

  pub(crate) fn upload_artifact_raw_bytes(&self, request: &CreateArtifactRequest, raw_data: Option<&[u8]>) {
    let task = self.new_task(request, raw_data);
    trace!("Enqueueing task: {:?}", task);
    let result = self.send_task_channel.send_blocking(task);
    if result.is_err() {
      panic!("Failed to send task to channel")
    }
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
  use log::trace;
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
    request.set_project_id(project_id.clone());
    //request.set_run_id(parent_data.get_run_id().clone());
    request.mut_artifact_id().set_uuid(new_uuid_proto());
    let artifact_data = request.mut_artifact_data();
    artifact_data.set_user_metadata(metadata);
    artifact_data.set_client_creation_time(time_now());

    let source_data_id = client.upload_artifact(&request, Some((&sphere).into()));

    client.shutdown().await?;
    Ok(())
  }
}
