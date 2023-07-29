use crate::task_handler::TaskHandler;
use crate::task_handler::TaskLoop;
use crate::upload_artifact_task::UploadArtifactTask;
use crate::upload_artifact_task::UploadArtifactTaskPayload;
use crate::uploaders::base_artifact_uploader::artifact_group_uploader_data_from_request;
use crate::uploaders::base_artifact_uploader::BaseArtifactUploaderBuilder;
use crate::uploaders::RunUploader;
use crate::util::decode_id_proto;
use crate::util::new_artifact_id;
use crate::util::time_now;
use crate::util::ClientError;
use crate::util::GenericError;
use crate::PublicArtifactId;
use crate::TokenGenerator;
use crate::UserMetadataBuilder;
use anyhow::anyhow;
use artifacts_api_rust_proto::public_global_id;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_ROOT_GROUP;
use artifacts_api_rust_proto::CreateArtifactRequest;
use artifacts_api_rust_proto::ProjectId;
use artifacts_api_rust_proto::PublicGlobalId;
use artifacts_api_rust_proto::StructuredData;
use protobuf::Message;
use std::io::Write;
use std::sync::Arc;
#[cfg(feature = "files")]
use tempfile::NamedTempFile;
#[cfg(feature = "files")]
use tempfile::TempDir;
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

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(
        ui_host: String,
        api_host: String,
        token: String,
        project_id: String,
    ) -> Result<Client, JsValue> {
        Client::new(ClientOptions {
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
                .init(),
        })
    }
}

impl Client {
    pub fn new(options: ClientOptions) -> Result<Self, GenericError> {
        let task_handler = Arc::new(TaskHandler {
            client: options.client.clone().unwrap_or_else(|| {
                let builder = reqwest::Client::builder().cookie_store(true);
                builder.build().expect("Failed to build reqwest client")
            }),
            token_generator: options.token_generator.clone(),
            host: options.api_host.clone().unwrap_or(API_HOST.to_string()),
        });

        let task_loop = Arc::new(TaskHandler::create_taskloop(task_handler.clone())?);

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
