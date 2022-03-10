use crate::base_artifact_uploader::ContextBehavior;
use crate::base_artifact_uploader::{time_now, BaseArtifactUploaderBuilder};
use crate::{RunStageUploader, RunUploader};
use artifacts_api_rust_proto::{
    ArtifactGroupUploaderData, CreateArtifactRequest, CreateRunRequest, CreateRunResponse,
};
use base64::decode;
use log::{debug, info};
use protobuf::{parse_from_bytes, Message};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::sync::Arc;
use tempfile::{NamedTempFile, TempDir};
use crate::google_token_generator::GoogleTokenGenerator;

#[derive(Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Client {
    tmp_dir: Arc<TempDir>,
    host: String,
    token_generator: Box<GoogleTokenGenerator>,
    project_id: String,
    client: reqwest::blocking::Client,
}

pub(crate) fn ffi_new_client(project_id: String) -> Box<Client> {
    env_logger::init();
    Box::new(Client::new(project_id))
}

#[cfg_attr(feature = "python", pymethods)]
impl Client {
    #[cfg(not(feature = "python"))]
    pub fn new(project_id: String) -> Self {
        Self::new_impl(project_id)
    }

    // TODO(doug): Figure out why this doesn't work: #[cfg_attr(feature = "python", new)]
    #[cfg(feature = "python")]
    #[new]
    pub fn new(project_id: String) -> Self {
        Self::new_impl(project_id)
    }

    pub fn create_run(&self) -> RunUploader {
        let mut request = CreateRunRequest::new();
        request.set_project_id(self.project_id.clone());
        request.mut_run_data().set_client_creation_time(time_now());

        let mut params = HashMap::new();
        params.insert("request", base64::encode(request.write_to_bytes().unwrap()));
        let res = self
            .client
            .post(format!("{}/create-run", self.host))
            .bearer_auth(self.token_generator.token())
            .form(&params)
            .send();
        let response = res.unwrap();
        if response.status().is_server_error() {
            debug!("{:?}", response);
            panic!("Server error {:?}", response)
        }

        let response_body = response.text().unwrap();
        let response: CreateRunResponse =
            parse_from_bytes(&decode(response_body).unwrap()).unwrap();
        let mut new_data = ArtifactGroupUploaderData::new();
        new_data.set_project_id(request.get_project_id().to_string());
        new_data.set_run_id(response.get_run_id().clone());
        new_data.set_id(response.get_root_stage_id().clone());
        RunUploader {
            base: BaseArtifactUploaderBuilder::default()
                .client(self.clone())
                .data(new_data)
                .context_behavior(ContextBehavior::Init)
                .init(),
            response,
        }
    }
}

impl Client {
    fn new_impl(project_id: String) -> Self {
        let host = env::var("OBS_HOST").unwrap_or("https://api.observation.tools".to_string());
        Client {
            tmp_dir: Arc::new(
                tempfile::Builder::new()
                    .prefix("observation_tools_")
                    .tempdir()
                    .unwrap(),
            ),
            host,
            token_generator: Box::new(GoogleTokenGenerator::new()),
            project_id,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(crate) fn ffi_create_run(&self) -> Box<RunUploader> {
        Box::new(self.create_run())
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

        let req_b64 = base64::encode(request.write_to_bytes().unwrap());
        let mut form = reqwest::blocking::multipart::Form::new().text("request", req_b64);
        if tmp_file.is_some() {
            form = form.file("raw_data", tmp_file.as_ref().unwrap()).unwrap();
        }

        let res = self
            .client
            .post(format!("{}/create-artifact", self.host))
            .bearer_auth(self.token_generator.token())
            .multipart(form)
            .send();
        let response = res.unwrap();
        if response.status().is_server_error() {
            debug!("{:?}", response);
            panic!("Server error")
        }

        if tmp_file.is_some() {
            tmp_file.unwrap().close().unwrap();
        }
    }
}
