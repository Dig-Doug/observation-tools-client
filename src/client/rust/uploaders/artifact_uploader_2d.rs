use crate::builders::Object2Builder;
use crate::builders::Object2Updater;
use crate::builders::SeriesBuilder;
use crate::builders::UserMetadataBuilder;
use crate::task_handle::Object2UpdaterTaskHandle;
use crate::task_handle::TaskHandle;
use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use crate::util::ClientError;
use crate::PublicSeriesIdTaskHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ArtifactUploader2d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader2d {
    pub fn upload_object2_js(
        &self,
        metadata: &UserMetadataBuilder,
        data: &Object2Builder,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        Ok(self
            .base
            .upload_raw(metadata, data.into(), data.series_point.as_ref())?
            .map_handle(|id| Object2Updater { id }))
    }

    pub fn update_object2_js(
        &self,
        artifact: &Object2Updater,
        data: &Object2Builder,
    ) -> Result<(), ClientError> {
        self.base
            .update_raw(&artifact.id, data.into(), data.series_point.as_ref())?;
        Ok(())
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series_js(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series)
    }
}

impl ArtifactUploader2d {
    pub fn upload_object2<M: Into<UserMetadataBuilder>, D: Into<Object2Builder>>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        let metadata = metadata.into();
        let data = data.into();
        self.upload_object2_js(&metadata, &data)
    }

    pub fn update_object2<D: Into<Object2Builder>>(
        &self,
        artifact: &Object2Updater,
        data: D,
    ) -> Result<(), ClientError> {
        let data = data.into();
        self.update_object2_js(&artifact, &data)
    }

    pub fn series<M: Into<UserMetadataBuilder>, D: Into<SeriesBuilder>>(
        &self,
        metadata: M,
        series: D,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        let metadata = metadata.into();
        let data = series.into();
        self.series_js(&metadata, &data)
    }
}
