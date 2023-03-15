use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::builders::Object2Builder;
use crate::user_metadata::UserMetadataBuilder;
use crate::util::ClientError;
use crate::PublicArtifactId;
use artifacts_api_rust_proto::StructuredData;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

pub trait Type2d {
    fn convert_2d_to_raw(&self) -> StructuredData;
}

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl ArtifactUploader2d {
    pub async fn upload_object2(
        &self,
        metadata: &UserMetadataBuilder,
        data: Object2Builder,
    ) -> Result<PublicArtifactId, ClientError> {
        self.upload(metadata, data).await
    }
}

impl ArtifactUploader2d {
    pub async fn upload(
        &self,
        metadata: &UserMetadataBuilder,
        data: impl Type2d,
    ) -> Result<PublicArtifactId, ClientError> {
        self.base
            .upload_raw(metadata, data.convert_2d_to_raw())
            .await
    }

    /*
    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
     */
}
