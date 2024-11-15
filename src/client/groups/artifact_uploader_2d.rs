use crate::artifacts::Object2;
use crate::artifacts::Series;
use crate::artifacts::Transform2;
use crate::artifacts::UserMetadata;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::Object2Updater;
use crate::task_handle::Object2UpdaterTaskHandle;
use crate::task_handle::TaskHandle;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use anyhow::Context;
use std::any::TypeId;
use wasm_bindgen::prelude::*;

/// An artifact group representing a 2-dimensional world. This group can only
/// contain [Object2](Object2) artifacts.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader2d {
    pub fn create_object2_js(
        &self,
        metadata: &UserMetadata,
        data: &Object2,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        self.create_object2(metadata.clone(), data.clone())
    }

    pub fn update_object2_js(
        &self,
        artifact: &Object2Updater,
        data: &Object2,
    ) -> Result<(), ClientError> {
        self.update_object2(artifact, data.clone())
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
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
        let metadata = metadata.into();
        let mut data = data.into();
        if TypeId::of::<D>() != TypeId::of::<Object2>() {
            // #implicit-transform
            data.add_transform(Transform2::identity());
        }

        Ok(self
            .base
            .upload_raw(
                metadata.clone(),
                data.clone()
                    .try_into()
                    .with_context(|| format!("Failed to parse object `{}`", metadata.name))?,
                data.series_point.as_ref(),
            )?
            .map_handle(|id| Object2Updater { id: id.id }))
    }

    pub fn update_object2<D: Into<Object2>>(
        &self,
        artifact: &Object2Updater,
        data: D,
    ) -> Result<(), ClientError> {
        let data = data.into();
        let series_point = data.series_point.clone();
        self.base
            .update_raw(artifact.id.clone(), data.try_into()?, series_point.as_ref())?;
        Ok(())
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
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        self.base.child_uploader_2d(metadata, None)
    }
}
