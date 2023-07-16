use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::client::UI_HOST;
use crate::run_id::RunId;
use crate::run_stage_uploader::RunStageUploader;
use crate::user_metadata::UserMetadataBuilder;
use crate::util::ClientError;
use crate::GenericArtifactUploader;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_RUN_STAGE;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct RunUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl RunUploader {
    pub fn run_id(&self) -> RunId {
        self.base.run_id()
    }

    pub fn viewer_url(&self) -> String {
        format!(
            "{}/project/{}/artifact/{}",
            self.base
                .client
                .options
                .ui_host
                .clone()
                .unwrap_or(UI_HOST.to_string()),
            self.base.client.options.project_id,
            self.base.id()
        )
    }

    pub async fn child_uploader(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<GenericArtifactUploader, ClientError> {
        self.base.child_uploader_async(metadata, None).await
    }

    pub fn create_initial_run_stage(&self, metadata: &UserMetadataBuilder) -> RunStageUploader {
        let mut request = self.base.base_create_artifact_request(metadata, None);
        request.mut_artifact_data().artifact_type = ARTIFACT_TYPE_RUN_STAGE.into();
        RunStageUploader {
            base: self.base.create_child_group_old(request, true),
        }
    }
}

impl RunUploader {
    #[cfg(feature = "cpp")]
    pub(crate) fn ffi_create_initial_run_stage(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Box<RunStageUploader> {
        Box::new(self.create_initial_run_stage(metadata))
    }
}
