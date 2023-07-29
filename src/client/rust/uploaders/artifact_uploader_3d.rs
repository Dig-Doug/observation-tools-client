use crate::builders::UserMetadataBuilder;
use crate::builders::{Object3Builder, PublicSeriesId, SeriesBuilder, Transform3Builder};
use crate::util::ClientError;
use crate::PublicArtifactId;
use artifacts_api_rust_proto::ArtifactType;


use crate::uploaders::base_artifact_uploader::BaseArtifactUploader;
use crate::uploaders::ArtifactUploader2d;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ArtifactUploader3d {
    pub(crate) base: BaseArtifactUploader,
}

#[wasm_bindgen]
impl ArtifactUploader3d {
    pub async fn upload_object3_js(
        &self,
        metadata: &UserMetadataBuilder,
        data: Object3Builder,
    ) -> Result<PublicArtifactId, ClientError> {
        self.base.upload_raw(metadata, (&data).into(), None).await
    }

    // TODO(doug): Where in the artifact hierarchy should series be defined?
    pub async fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesId, ClientError> {
        self.base.series(metadata, series).await
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
}

impl ArtifactUploader3d {
    pub async fn upload_object3<M: Into<UserMetadataBuilder>, D: Into<Object3Builder>>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactId, ClientError> {
        self.base
            .upload_raw(&(metadata.into()), (&data.into()).into(), None)
            .await
    }

    pub async fn child_uploader_2d<M: Into<UserMetadataBuilder>, T: Into<Transform3Builder>>(
        &self,
        metadata: M,
        to_3d_transform: T,
    ) -> Result<ArtifactUploader2d, ClientError> {
        let mut request = self
            .base
            .base_create_artifact_request(&metadata.into(), None);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_2D_IN_3D_GROUP.into();
        let transform: Transform3Builder = to_3d_transform.into();
        artifact_data.mut_map_2d_to_3d().to_3d_transform = Some(transform.proto).into();
        Ok(ArtifactUploader2d {
            base: self.base.create_child_group(request).await?,
        })
    }
}
