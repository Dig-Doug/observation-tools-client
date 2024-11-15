use crate::artifacts::Transform3;
use crate::artifacts::UserMetadata;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use crate::PublicArtifactIdTaskHandle;
use crate::RunUploaderTaskHandle;
use pyo3::pyclass;
use pyo3::pymethods;
use wasm_bindgen::prelude::*;

/// The "root" artifact group for a run.
#[wasm_bindgen]
#[pyclass]
#[derive(Debug, Clone)]
pub struct RunUploader {
    pub(crate) base: BaseArtifactUploader,
    pub(crate) handle: PublicArtifactIdTaskHandle,
}

#[wasm_bindgen]
#[pymethods]
impl RunUploader {
    pub fn viewer_url(&self) -> String {
        todo!("impl");
        /*
        format!(
            "{}/project/{}/artifact/{}",
            self.base
                .client
                .inner
                .options
                .ui_host
                .clone()
                .unwrap_or(UI_HOST.to_string()),
            bs58::encode(self.base.project_global_id().encode_to_vec()).into_string(),
            bs58::encode(self.base.global_id().encode_to_vec()).into_string(),
        )
         */
    }

    pub fn child_uploader_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.child_uploader(metadata.clone())
    }
}

impl RunUploader {
    pub fn child_uploader<M: Into<UserMetadata>>(
        &self,
        metadata: M,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.base.child_uploader(metadata, None)
    }

    pub fn child_uploader_2d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        self.base.child_uploader_2d(metadata, None)
    }

    pub fn child_uploader_3d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata, Transform3::identity(), None)
    }
}
