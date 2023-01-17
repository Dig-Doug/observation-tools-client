use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::run_stage_uploader::RunStageUploader;
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_RUN_STAGE;
#[cfg(feature = "python")]
use pyo3::prelude::*;

use artifacts_api_rust_proto::CreateRunResponse;

#[cfg_attr(feature = "python", pyclass)]
pub struct RunUploader {
    pub(crate) base: BaseArtifactUploader,
    pub(crate) response: CreateRunResponse,
}

#[cfg_attr(feature = "python", pymethods)]
impl RunUploader {
    pub fn viewer_url(&self) -> &str {
        &self.response.viewer_url
    }

    pub fn create_initial_run_stage(&self, metadata: &UserMetadataBuilder) -> RunStageUploader {
        let mut request = self.base.create_base_child_group_request(metadata);
        request
            .mut_artifact_data()
            .set_artifact_type(ARTIFACT_TYPE_RUN_STAGE);
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
