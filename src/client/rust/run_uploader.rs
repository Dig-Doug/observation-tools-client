use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::run_stage_uploader::RunStageUploader;
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_RUN_STAGE;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

use crate::run_id::RunId;
use artifacts_api_rust_proto::CreateRunResponse;

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
            "{}/project/{}/run/{}",
            self.base.client.options.ui_host,
            self.base.client.options.project_id,
            self.base.id()
        )
    }

    pub fn create_initial_run_stage(&self, metadata: &UserMetadataBuilder) -> RunStageUploader {
        let mut request = self.base.create_base_child_group_request(metadata);
        request.mut_artifact_data().artifact_type = ARTIFACT_TYPE_RUN_STAGE.into();
        RunStageUploader {
            base: self.base.create_child_group(request, true),
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
