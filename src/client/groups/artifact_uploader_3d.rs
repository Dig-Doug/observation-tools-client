use crate::artifacts::Object2;
use crate::artifacts::Object3;
use crate::artifacts::Series;
use crate::artifacts::Transform3;
use crate::artifacts::UserMetadata;
use crate::groups::base_artifact_uploader::BaseArtifactUploader;
use crate::groups::ArtifactUploader2d;
use crate::task_handle::TaskHandle;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use observation_tools_common::artifact::ArtifactType;
use observation_tools_common::artifact::Map2dTo3dData;
use std::any::TypeId;
use wasm_bindgen::prelude::*;

/// An artifact group representing a 3-dimensional world. This group can only
/// contain [Object3](Object3) artifacts.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ArtifactUploader3d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader3d {
    pub fn create_object3_js(
        &self,
        metadata: &UserMetadata,
        data: Object3,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.create_object3(metadata.clone(), data)
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub fn series(
        &self,
        metadata: &UserMetadata,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        self.base.series(metadata.clone(), series)
    }

    pub fn child_uploader_3d(
        &self,
        metadata: &UserMetadata,
        base_transform: Transform3,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        self.base
            .child_uploader_3d(metadata.clone(), base_transform, None)
    }
}

impl ArtifactUploader3d {
    pub fn create_object3<M: Into<UserMetadata>, D: Into<Object3> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let mut object3 = data.into();
        if TypeId::of::<D>() != TypeId::of::<Object2>() {
            // #implicit-transform
            object3.add_transform(Transform3::identity());
        }
        self.base.upload_raw(metadata, object3.try_into()?, None)
    }

    pub fn child_uploader_2d<M: Into<UserMetadata>, T: Into<Transform3>>(
        &self,
        metadata: M,
        to_3d_transform: T,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        let request = self.base.base_create_artifact_request(
            metadata,
            ArtifactType::Group2dIn3d(Map2dTo3dData {
                to_3d_transform: to_3d_transform.into(),
            }),
            None,
        );
        Ok(self
            .base
            .create_child_group(request)?
            .map_handle(|base| ArtifactUploader2d { base }))
    }
}
