use crate::artifacts::Transform3Builder;
use crate::artifacts::UserMetadataBuilder;
use crate::client::UI_HOST;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::run_id::RunId;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use protobuf::Message;
use wasm_bindgen::prelude::*;

/// The "root" artifact group for a run.
#[wasm_bindgen]
#[derive(Debug, Clone)]
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

    pub fn child_uploader_js(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.child_uploader(metadata.clone())
    }
}

impl RunUploader {
    pub fn child_uploader<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.base.child_uploader(metadata, None)
    }

    pub fn child_uploader_2d<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        self.base.child_uploader_2d(metadata, None)
    }

    pub fn child_uploader_3d<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata, Transform3Builder::identity().proto, None)
    }
}
