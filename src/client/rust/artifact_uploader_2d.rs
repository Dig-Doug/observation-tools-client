use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::StructuredData;
#[cfg(feature = "python")]
use pyo3::prelude::*;

pub trait Type2d {
    fn convert_2d_to_raw(&self) -> StructuredData;
}

#[cfg_attr(feature = "python", pyclass)]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
impl ArtifactUploader2d {}

impl ArtifactUploader2d {
    pub fn upload(&self, metadata: &UserMetadataBuilder, data: impl Type2d) -> String {
        self.base.upload_raw(metadata, data.convert_2d_to_raw())
    }

    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
}
