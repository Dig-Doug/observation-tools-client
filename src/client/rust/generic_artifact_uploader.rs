use log::trace;
use protobuf::Message;
use crate::artifact_uploader_2d::ArtifactUploader2d;
use crate::artifact_uploader_3d::ArtifactUploader3d;
use crate::base_artifact_uploader::BaseArtifactUploader;
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::{StructuredData, Transform3};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use crate::api::Image2Builder;

#[cfg_attr(feature = "python", pyclass)]
pub struct GenericArtifactUploader {
  pub(crate) base: BaseArtifactUploader,
}

#[cfg_attr(feature = "python", pymethods)]
impl GenericArtifactUploader {
  pub fn child_uploader(&self, metadata: &UserMetadataBuilder) -> GenericArtifactUploader {
    self.base.child_uploader(metadata)
  }

  pub fn child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
    self.base.child_uploader_2d(metadata)
  }

  pub fn upload_image2(&self, metadata: &UserMetadataBuilder, data: &Image2Builder) -> String {
    self.upload(metadata, data)
  }
}

impl GenericArtifactUploader {
  pub(crate) fn ffi_child_uploader(
    &self,
    metadata: &UserMetadataBuilder,
  ) -> Box<GenericArtifactUploader> {
    Box::new(self.child_uploader(metadata))
  }

  pub(crate) fn ffi_child_uploader_2d(
    &self,
    metadata: &UserMetadataBuilder,
  ) -> Box<ArtifactUploader2d> {
    Box::new(self.child_uploader_2d(metadata))
  }

  pub fn child_uploader_3d(
    &self,
    metadata: &UserMetadataBuilder,
    base_transform: Transform3,
  ) -> ArtifactUploader3d {
    self.base.child_uploader_3d(metadata, base_transform)
  }

  pub fn ffi_child_uploader_3d(
    &self,
    metadata: &UserMetadataBuilder,
    transform3_bytes: &[u8],
  ) -> Box<ArtifactUploader3d> {
    let transform = Transform3::parse_from_bytes(transform3_bytes).unwrap();
    Box::new(self.child_uploader_3d(metadata, transform))
  }

  pub(crate) fn ffi_upload(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
    self.base.upload_raw_bytes(metadata, data)
  }

  pub fn upload(&self, metadata: &UserMetadataBuilder, data: impl Into<StructuredData>) -> String {
    self.base.upload_raw(metadata, data.into())
  }
}
