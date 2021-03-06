use crate::artifact_uploader_2d::ArtifactUploader2d;
use crate::artifact_uploader_3d::ArtifactUploader3d;
use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::generic_artifact_uploader::GenericArtifactUploader;
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::Transform3;
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
pub struct RunStageUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
impl RunStageUploader {
    pub fn child_uploader(&self, metadata: &UserMetadataBuilder) -> GenericArtifactUploader {
        self.base.child_uploader(metadata)
    }

    pub fn child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
        self.base.child_uploader_2d(metadata)
    }
}

impl RunStageUploader {
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
        self.base.child_uploader_3d(metadata, base_transform)
    }

    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
}
