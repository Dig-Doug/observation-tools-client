use crate::artifacts::Transform3;
use crate::artifacts::UserMetadata;
use crate::child_uploader_2d_impl;
use crate::child_uploader_3d_impl;
use crate::child_uploader_impl;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::groups::GenericArtifactUploader;
use crate::util::ClientError;
use crate::PublicArtifactIdTaskHandle;
use observation_tools_common::artifacts::Object1;
use pyo3::pyclass;
use pyo3::pymethods;
use wasm_bindgen::prelude::*;

/// The "root" artifact group for a run.
#[wasm_bindgen]
#[pyclass]
#[derive(Debug, Clone)]
pub struct RunUploader {
    pub(crate) base: BaseArtifactUploader,
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

    #[pyo3(name = "child_uploader")]
    pub fn child_uploader_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<GenericArtifactUploader, ClientError> {
        self.child_uploader(metadata.clone())
    }

    #[pyo3(name = "create_object1")]
    pub fn create_object1_js(
        &self,
        metadata: UserMetadata,
        data: Object1,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.base.create_object1(metadata, data)
    }
}

impl RunUploader {
    child_uploader_impl!();
    child_uploader_2d_impl!();
    child_uploader_3d_impl!();

    pub fn create_object1<M: Into<UserMetadata>, D: Into<Object1> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.base.create_object1(metadata, data)
    }
}
