use crate::artifacts::Object2;
use crate::artifacts::Series;
use crate::artifacts::Transform2;
use crate::artifacts::UserMetadata;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::Object2Updater;
use crate::task_handle::Object2UpdaterTaskHandle;
use crate::task_handle::TaskHandle;
use crate::util::ClientError;
use crate::PublicSeriesIdTaskHandle;
use anyhow::Context;
use pyo3::pyclass;
use std::any::TypeId;
use wasm_bindgen::prelude::*;

/// An artifact group representing a 2-dimensional world. This group can only
/// contain [Object2](Object2) artifacts.
#[wasm_bindgen]
#[pyclass]
#[derive(Debug, Clone)]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
#[pyo3::pymethods]
impl ArtifactUploader2d {
    #[pyo3(name = "create_object2")]
    pub fn create_object2_js(
        &self,
        metadata: &UserMetadata,
        data: &Object2,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        self.create_object2(metadata.clone(), data.clone())
    }

    #[pyo3(name = "update_object2")]
    pub fn update_object2_js(
        &self,
        artifact: &Object2Updater,
        data: &Object2,
    ) -> Result<(), ClientError> {
        self.update_object2(artifact, data.clone())
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    #[pyo3(name = "series")]
    pub fn series_js(
        &self,
        metadata: &UserMetadata,
        series: &Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.series(metadata.clone(), series.clone())
    }
}

impl ArtifactUploader2d {
    pub fn create_object2<M: Into<UserMetadata>, D: Into<Object2> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        self.base.create_object2(metadata, data)
    }

    pub fn update_object2<D: Into<Object2>>(
        &self,
        artifact: &Object2Updater,
        data: D,
    ) -> Result<(), ClientError> {
        self.base.update_object2(artifact, data)
    }

    pub fn series<M: Into<UserMetadata>, D: Into<Series>>(
        &self,
        metadata: M,
        series: D,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series.into())
    }

    pub fn child_uploader_2d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
    ) -> Result<ArtifactUploader2d, ClientError> {
        self.base.child_uploader_2d(metadata, None)
    }
}
