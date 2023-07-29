use crate::builders::UserMetadataBuilder;
use crate::builders::{PublicSeriesId, SeriesBuilder, Transform3Builder};
use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use crate::uploaders::{ArtifactUploader2d, ArtifactUploader3d};
use crate::util::ClientError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GenericArtifactUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl GenericArtifactUploader {
    pub async fn child_uploader(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<GenericArtifactUploader, ClientError> {
        self.base.child_uploader(metadata, None).await
    }

    pub async fn child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<ArtifactUploader2d, ClientError> {
        self.base.child_uploader_2d(metadata, None).await
    }

    pub async fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3Builder,
    ) -> Result<ArtifactUploader3d, ClientError> {
        self.base
            .child_uploader_3d(metadata, base_transform.proto, None)
            .await
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
