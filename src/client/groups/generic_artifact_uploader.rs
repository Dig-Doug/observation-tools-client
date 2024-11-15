use crate::artifacts::Series;
use crate::artifacts::Transform3;
use crate::artifacts::UserMetadata;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use wasm_bindgen::prelude::*;

/// An artifact group that can contain any type of artifact and create
/// specialized child groups.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct GenericArtifactUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl GenericArtifactUploader {
    pub fn child_uploader_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        self.child_uploader(metadata.clone())
    }

    pub fn child_uploader_2d_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        self.child_uploader_2d(metadata.clone())
    }

    pub fn child_uploader_3d_js(
        &self,
        metadata: &UserMetadata,
        base_transform: Transform3,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.child_uploader_3d(metadata.clone(), base_transform)
    }

    pub fn series_js(
        &self,
        metadata: &UserMetadata,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.series(metadata.clone(), series)
    }
}

impl GenericArtifactUploader {
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
        base_transform: Transform3,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base.child_uploader_3d(metadata, base_transform, None)
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series)
    }
}
