use std::cell::RefCell;
use crate::base_artifact_uploader::ContextBehavior;
use crate::base_artifact_uploader::{time_now, BaseArtifactUploaderBuilder};
use crate::{RunStageUploader, RunUploader};
use artifacts_api_rust_proto::{
    ArtifactGroupUploaderData, CreateArtifactRequest, CreateRunRequest, CreateRunResponse,
};
use base64::decode;
use log::{debug, trace};
use protobuf::{parse_from_bytes, Message};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::sync::Arc;
use reqwest::multipart::Part;
use reqwest::{RequestBuilder, Response};
use tempfile::{NamedTempFile, TempDir};
use tokio::runtime::Runtime;
use crate::google_token_generator::{GenericError, GoogleTokenGenerator};
use tokio_util::codec::{BytesCodec, FramedRead};
use crate::aws_sign_in::SignInWithAwsTokenGenerator;


#[derive(Clone)]
enum TokenGenerator {
    GoogleToken { i: GoogleTokenGenerator },
    SignInWithAws { i: SignInWithAwsTokenGenerator },
}

#[derive(Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Client {
    tmp_dir: Arc<TempDir>,
    host: String,
    token_generator: TokenGenerator,
    project_id: String,
    client: reqwest::Client,
    runtime: Arc<Runtime>,
}

fn default_reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .use_rustls_tls()
        .build().expect("Failed to build reqwest client")
}

#[cfg(not(feature = "python"))]
pub(crate) fn ffi_new_client(project_id: String) -> Box<Client> {
    env_logger::init();
    Box::new(Client::new(project_id, default_reqwest_client()))
}

#[cfg_attr(feature = "python", pymethods)]
impl Client {
    // TODO(doug): Figure out why this doesn't work: #[cfg_attr(feature = "python", new)]
    #[cfg(feature = "python")]
    #[new]
    pub fn new(project_id: String) -> Self {
        env_logger::init();
        Self::new_impl(project_id, default_reqwest_client())
    }

    pub fn create_run_blocking(&self) -> RunUploader {
        self.runtime.block_on(self.create_run()).unwrap()
    }
}

impl Client {
    #[cfg(not(feature = "python"))]
    pub fn new(project_id: String, client: reqwest::Client) -> Self {
        Self::new_impl(project_id, client)
    }

    fn new_impl(project_id: String, client: reqwest::Client) -> Self {
        let host = env::var("OBS_HOST").unwrap_or("https://api.observation.tools".to_string());
        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap());
        let token_generator = if env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
            TokenGenerator::GoogleToken {
                i: GoogleTokenGenerator {
                    client: client.clone(),
                }
            }
        } else {
            TokenGenerator::SignInWithAws {
                i: SignInWithAwsTokenGenerator {
                    runtime: runtime.clone(),
                    client: client.clone(),
                }
            }
        };
        Client {
            tmp_dir: Arc::new(
                tempfile::Builder::new()
                    .prefix("observation_tools_")
                    .tempdir()
                    .unwrap(),
            ),
            host,
            token_generator,
            project_id,
            client,
            runtime,
        }
    }

    pub async fn create_run(&self) -> Result<RunUploader, GenericError> {
        let response = self.create_run_request().await?.send().await?;
        self.parse_create_run_response(response).await
    }

    pub async fn create_run_request(&self) -> Result<RequestBuilder, GenericError> {
        let mut request = CreateRunRequest::new();
        request.set_project_id(self.project_id.clone());
        request.mut_run_data().set_client_creation_time(time_now());

        let mut params = HashMap::new();
        params.insert("request", base64::encode(request.write_to_bytes().unwrap()));
        let token = self.token().await?;
        let request_builder = self
            .client
            .post(format!("{}/create-run", self.host))
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
        new_data.set_project_id(self.project_id.clone());
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

    pub(crate) fn ffi_deserialize_run_stage(&self, serialized: String) -> Box<RunStageUploader> {
        Box::new(self.deserialize_run_stage(serialized))
    }

    pub(crate) fn upload_artifact(&self, request: &CreateArtifactRequest, raw_data: Option<&[u8]>) {
        let tmp_file = if let Some(raw_data_slice) = raw_data {
            // TODO(doug): Consider using a spooled tempfile
            let mut tmp_file = NamedTempFile::new_in(&*self.tmp_dir).unwrap();
            tmp_file.write_all(raw_data_slice).unwrap();
            Some(tmp_file)
        } else {
            None
        };
        
        trace!("Uploading artifact: {:?}", request);

        let req_b64 = base64::encode(request.write_to_bytes().unwrap());
        let this = self.clone();
        let task = async move {
            let mut form = reqwest::multipart::Form::new().text("request", req_b64);
            if tmp_file.is_some() {
                let f = tmp_file.as_ref().unwrap();
                let file = tokio::fs::File::open(f).await?;
                let reader = reqwest::Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
                form = form.part("raw_data", Part::stream(reader));
            }

            let token = this.token().await?;
            let response = this
                .client
                .post(format!("{}/create-artifact", this.host))
                .bearer_auth(token)
                .multipart(form)
                .send()
                .await?;

            if response.status().is_server_error() {
                debug!("{:?}", response);
                panic!("Server error")
            }

            Ok::<(), GenericError>(())
        };
        self.runtime.spawn(task);
    }

    async fn token(&self) -> Result<String, std::io::Error> {
        match &self.token_generator {
            TokenGenerator::GoogleToken { i } => i.token().await,
            TokenGenerator::SignInWithAws { i } => i.token().await,
        }
    }
}