use crate::artifacts::Series;
use crate::artifacts::Transform3;
use crate::artifacts::UserMetadata;
use crate::child_uploader_2d_impl;
use crate::child_uploader_3d_impl;
use crate::child_uploader_impl;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::util::ClientError;
use crate::PublicSeriesIdTaskHandle;
use pyo3::pyclass;
use wasm_bindgen::prelude::*;

/// An artifact group that can contain any type of artifact and create
/// specialized child groups.
#[wasm_bindgen]
#[pyclass]
#[derive(Debug, Clone)]
pub struct GenericArtifactUploader {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
#[pyo3::pymethods]
impl GenericArtifactUploader {
    #[pyo3(name = "child_uploader")]
    pub fn child_uploader_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<GenericArtifactUploader, ClientError> {
        self.child_uploader(metadata.clone())
    }

    #[pyo3(name = "child_uploader_2d")]
    pub fn child_uploader_2d_js(
        &self,
        metadata: &UserMetadata,
    ) -> Result<ArtifactUploader2d, ClientError> {
        self.child_uploader_2d(metadata.clone())
    }

    #[pyo3(name = "child_uploader_3d")]
    pub fn child_uploader_3d_js(
        &self,
        metadata: &UserMetadata,
        base_transform: Transform3,
    ) -> Result<ArtifactUploader3d, ClientError> {
        self.child_uploader_3d(metadata.clone(), base_transform)
    }

    #[pyo3(name = "series")]
    pub fn series_js(
        &self,
        metadata: &UserMetadata,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.series(metadata.clone(), series)
    }
}

impl GenericArtifactUploader {
    child_uploader_impl!();
    child_uploader_2d_impl!();
    child_uploader_3d_impl!();

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series)
    }
}
