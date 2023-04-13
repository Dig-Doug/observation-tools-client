use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::builders::Object3Builder;
use crate::user_metadata::UserMetadataBuilder;
use crate::util::ClientError;
use crate::ArtifactUploader2d;
use crate::PublicArtifactId;
use artifacts_api_rust_proto::ArtifactType;
use artifacts_api_rust_proto::StructuredData;
use artifacts_api_rust_proto::Transform3;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

pub trait Type3d {
    fn convert_3d_to_raw(&self) -> StructuredData;
}

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct ArtifactUploader3d {
    pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl ArtifactUploader3d {
    pub async fn upload_object3(
        &self,
        metadata: &UserMetadataBuilder,
        data: Object3Builder,
    ) -> Result<PublicArtifactId, ClientError> {
        self.upload(metadata, data).await
    }
}

impl ArtifactUploader3d {
    pub async fn upload(
        &self,
        metadata: &UserMetadataBuilder,
        data: impl Type3d,
    ) -> Result<PublicArtifactId, ClientError> {
        self.base
            .upload_raw(metadata, data.convert_3d_to_raw())
            .await
    }

    pub fn child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
        to_3d_transform: Transform3,
    ) -> ArtifactUploader2d {
        let mut request = self.base.create_base_child_group_request(metadata);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_2D_IN_3D_GROUP.into();
        artifact_data.mut_map_2d_to_3d().to_3d_transform = Some(to_3d_transform).into();
        ArtifactUploader2d {
            base: self.base.create_child_group_old(request, false),
        }
    }

    /*
    pub(crate) fn ffi_child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
        to_3d_transform_bytes: &[u8],
    ) -> Box<ArtifactUploader2d> {
        let to_3d_transform = Transform3::parse_from_bytes(to_3d_transform_bytes).unwrap();
        Box::new(self.child_uploader_2d(metadata, to_3d_transform))
    }

    pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
        self.base.upload_raw_bytes(metadata, data)
    }
     */
}
