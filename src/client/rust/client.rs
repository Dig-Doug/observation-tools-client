use crate::artifacts::UserMetadataBuilder;
use crate::generated::public_global_id;
use crate::generated::ArtifactType::ARTIFACT_TYPE_ROOT_GROUP;
use crate::generated::CreateArtifactRequest;
use crate::generated::ProjectId;
use crate::generated::PublicGlobalId;
use crate::generated::StructuredData;
use crate::groups::base_artifact_uploader::artifact_group_uploader_data_from_request;
use crate::groups::base_artifact_uploader::BaseArtifactUploaderBuilder;
use crate::groups::RunUploader;
use crate::task_handle::TaskHandle;
use crate::task_loop::TaskHandler;
use crate::task_loop::TaskLoop;
use crate::util::decode_id_proto;
use crate::util::new_artifact_id;
use crate::util::time_now;
use crate::util::ClientError;
use crate::PublicArtifactIdTaskHandle;
use crate::RunUploaderTaskHandle;
use crate::TokenGenerator;
use anyhow::anyhow;
use protobuf::Message;
use std::sync::Arc;
use tracing::trace;
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
            token_generator: TokenGenerator::OAuth2DeviceCodeFlow,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) options: ClientOptions,
    pub(crate) project_id: ProjectId,
    task_loop: Arc<TaskLoop>,
}

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

    pub async fn shutdown(self) -> Result<(), ClientError> {
        trace!("Shutting down client");
        self.task_loop.shutdown().await;
        trace!("Finished shutting down client");
        Ok(())
    }

    pub fn create_run_js(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<RunUploaderTaskHandle, ClientError> {
        self.create_run(metadata.clone())
    }
}

impl Client {
    pub fn new(project_id: String, options: ClientOptions) -> Result<Self, ClientError> {
        trace!("Creating client");
        let task_handler = Arc::new(TaskHandler {
            client: options.reqwest_client.clone().unwrap_or_else(|| {
                let builder = reqwest::Client::builder().cookie_store(true);
                builder.build().expect("Failed to build reqwest client")
            }),
            token_generator: options.token_generator.clone(),
            host: options.api_host.clone().unwrap_or(API_HOST.to_string()),
        });

        let task_loop =
            Arc::new(TaskLoop::new(task_handler.clone()).map_err(ClientError::from_string)?);

        let proto: PublicGlobalId =
            decode_id_proto(&project_id).map_err(ClientError::from_string)?;
        let project_id = match proto.data {
            Some(public_global_id::Data::ProjectId(project_id)) => project_id,
            _ => Err(anyhow!("Invalid project id: {}", project_id))?,
        };

        let client = Client {
            options,
            project_id,
            task_loop,
        };
        Ok(client)
    }

    pub(crate) fn upload_artifact(
        &self,
        request: &CreateArtifactRequest,
        structured_data: Option<StructuredData>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let bytes = structured_data.and_then(|s| s.write_to_bytes().ok());
        self.upload_artifact_raw_bytes(request, bytes.as_ref().map(|b| b.as_slice()))
    }

    pub(crate) fn upload_artifact_raw_bytes(
        &self,
        request: &CreateArtifactRequest,
        raw_data: Option<&[u8]>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.task_loop.submit_task(request, raw_data)
    }

    pub fn create_run<M: Into<UserMetadataBuilder>>(
        &self,
        into_metadata: M,
    ) -> Result<RunUploaderTaskHandle, ClientError> {
        let mut request = CreateArtifactRequest::new();
        request.project_id = Some(self.project_id.clone()).into();
        request.artifact_id = Some(new_artifact_id()).into();
        request.run_id.mut_or_insert_default().id = request.artifact_id.clone();
        let group_data = request.mut_artifact_data();
        let metadata = into_metadata.into();
        group_data.user_metadata = Some(metadata.proto.clone()).into();
        group_data.artifact_type = ARTIFACT_TYPE_ROOT_GROUP.into();
        group_data.client_creation_time = Some(time_now()).into();
        Ok(self
            .upload_artifact(&request, None)?
            .map_handle(|_result| RunUploader {
                base: BaseArtifactUploaderBuilder::default()
                    .client(self.clone())
                    .data(artifact_group_uploader_data_from_request(&request))
                    .init(),
            }))
    }
}
