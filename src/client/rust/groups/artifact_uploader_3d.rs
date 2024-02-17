use crate::artifacts::Object2Builder;
use crate::artifacts::Object3Builder;
use crate::artifacts::SeriesBuilder;
use crate::artifacts::Transform3Builder;
use crate::artifacts::UserMetadataBuilder;
use crate::generated::ArtifactType;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::ArtifactUploader2d;
use crate::task_handle::TaskHandle;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use std::any::TypeId;
use wasm_bindgen::prelude::*;

/// An artifact group representing a 3-dimensional world. This group can only
/// contain [Object3](Object3Builder) artifacts.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct ArtifactUploader3d {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ArtifactUploader3d {
    pub fn create_object3_js(
        &self,
        metadata: &UserMetadataBuilder,
        data: Object3Builder,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.create_object3(metadata.clone(), data)
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata.clone(), series)
    }

    pub fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3Builder,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata.clone(), base_transform.proto, None)
    }
}

impl ArtifactUploader3d {
    pub fn create_object3<M: Into<UserMetadataBuilder>, D: Into<Object3Builder> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let mut object3 = data.into();
        if TypeId::of::<D>() != TypeId::of::<Object2Builder>() {
            // #implicit-transform
            object3.add_transform(&Transform3Builder::identity());
        }
        self.base.upload_raw(metadata, object3.try_into()?, None)
    }

    pub fn child_uploader_2d<M: Into<UserMetadataBuilder>, T: Into<Transform3Builder>>(
        &self,
        metadata: M,
        to_3d_transform: T,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        let mut request = self.base.base_create_artifact_request(metadata, None);
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
