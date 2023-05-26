use crate::artifact_uploader_2d::ArtifactUploader2d;
use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::generic_artifact_uploader::GenericArtifactUploader;
use crate::user_metadata::UserMetadataBuilder;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct RunStageUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl RunStageUploader {
    pub fn child_uploader(&self, metadata: &UserMetadataBuilder) -> GenericArtifactUploader {
        self.base.child_uploader_old(metadata)
    }

    pub fn child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
        self.base.child_uploader_2d_old(metadata)
    }
}

impl RunStageUploader {
    /*
    pub(crate) fn ffi_child_uploader(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Box<GenericArtifactUploader> {
        Box::new(self.child_uploader(metadata))
    }

    pub(crate) fn ffi_child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Box<ArtifactUploader2d> {
        Box::new(self.child_uploader_2d(metadata))
    }

    pub fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3,
    ) -> ArtifactUploader3d {
        self.base.child_uploader_3d_old(metadata, base_transform)
    }

    pub fn upload(&self, metadata: &UserMetadataBuilder, data: StructuredData) -> String {
        self.base.upload_raw(metadata, data)
    }

    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
     */
}
