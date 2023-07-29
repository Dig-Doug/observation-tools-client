use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::builders::UserMetadataBuilder;
use crate::client::UI_HOST;
use crate::run_id::RunId;
use crate::run_stage_uploader::RunStageUploader;
use crate::util::ClientError;
use crate::GenericArtifactUploader;
use artifacts_api_rust_proto::ArtifactType::ARTIFACT_TYPE_RUN_STAGE;

use protobuf::Message;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct RunUploader {
    pub(crate) base: BaseArtifactUploader,
}

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
            bs58::encode(self.base.project_global_id().write_to_bytes().unwrap()).into_string(),
            bs58::encode(self.base.global_id().write_to_bytes().unwrap()).into_string(),
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
