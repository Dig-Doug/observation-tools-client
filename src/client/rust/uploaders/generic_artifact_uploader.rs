use crate::builders::SeriesBuilder;
use crate::builders::Transform3Builder;
use crate::builders::UserMetadataBuilder;
use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct GenericArtifactUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl GenericArtifactUploader {
    pub fn child_uploader(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.base.child_uploader(metadata, None)
    }

    pub fn child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        self.base.child_uploader_2d(metadata, None)
    }

    pub fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3Builder,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata, base_transform.proto, None)
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series)
    }
}
