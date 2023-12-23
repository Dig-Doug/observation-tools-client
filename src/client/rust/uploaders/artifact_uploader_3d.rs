use crate::builders::Object3Builder;
use crate::builders::SeriesBuilder;
use crate::builders::Transform2Builder;
use crate::builders::Transform3Builder;
use crate::builders::UserMetadataBuilder;
use crate::generated::ArtifactType;
use crate::task_handle::TaskHandle;
use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use crate::uploaders::ArtifactUploader2d;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ArtifactUploader3d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader3d {
    pub fn upload_object3_js(
        &self,
        metadata: &UserMetadataBuilder,
        data: Object3Builder,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.base.upload_raw(metadata, (&data).try_into()?, None)
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata, series)
    }

    pub fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3Builder,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata, base_transform.proto, None)
    }
}

impl ArtifactUploader3d {
    pub fn upload_object3<M: Into<UserMetadataBuilder>, D: Into<Object3Builder>>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let mut object3 = data.into();
        // #implicit-transform
        object3.add_transform(&Transform3Builder::identity());
        self.base
            .upload_raw(&(metadata.into()), (&object3).try_into()?, None)
    }

    pub fn child_uploader_2d<M: Into<UserMetadataBuilder>, T: Into<Transform3Builder>>(
        &self,
        metadata: M,
        to_3d_transform: T,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        let mut request = self
            .base
            .base_create_artifact_request(&metadata.into(), None);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_2D_IN_3D_GROUP.into();
        let transform: Transform3Builder = to_3d_transform.into();
        artifact_data.mut_map_2d_to_3d().to_3d_transform = Some(transform.proto).into();
        Ok(self
            .base
            .create_child_group(request)?
            .map_handle(|base| ArtifactUploader2d { base }))
    }
}
