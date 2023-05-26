use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::builders::{
    Object2Builder, Object2Updater, PublicSeriesId, SeriesBuilder, SeriesPointBuilder,
};
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

#[wasm_bindgen]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader2d {
    pub async fn upload_object2(
        &self,
        metadata: &UserMetadataBuilder,
        data: &Object2Builder,
    ) -> Result<Object2Updater, ClientError> {
        Ok(Object2Updater {
            id: self
                .base
                .upload_raw(
                    metadata,
                    data.convert_2d_to_raw(),
                    data.series_point.as_ref(),
                )
                .await?,
        })
    }

    pub async fn update_object2(
        &self,
        artifact: &Object2Updater,
        data: &Object2Builder,
    ) -> Result<(), ClientError> {
        self.base
            .update_raw(
                &artifact.id,
                data.convert_2d_to_raw(),
                data.series_point.as_ref(),
            )
            .await?;
        Ok(())
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub async fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesId, ClientError> {
        self.base.series(metadata, series).await
    }
}

impl ArtifactUploader2d {
    /*
    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
     */
}
