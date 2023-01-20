use crate::artifact_uploader_2d::ArtifactUploader2d;
use crate::artifact_uploader_3d::ArtifactUploader3d;
use crate::client::Client;
use crate::generic_artifact_uploader::GenericArtifactUploader;
use crate::uploader_stack::{init_uploader_stack, pop_uploader, push_uploader};
use crate::user_metadata::UserMetadataBuilder;
use artifacts_api_rust_proto::{
  ArtifactGroupUploaderData, ArtifactType, ArtifactUserMetadata, CreateArtifactRequest,
  StructuredData, Transform3,
};
use derive_builder::Builder;
use protobuf::Message;
use crate::api::new_artifact_id;
use crate::util::{time_now};

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum ContextBehavior {
  Disabled,
  Init,
  PushPop,
}

#[derive(Builder)]
pub(crate) struct BaseArtifactUploader {
  pub(crate) client: Client,
  data: ArtifactGroupUploaderData,
  context_behavior: ContextBehavior,
}

impl BaseArtifactUploaderBuilder {
  pub fn init(&mut self) -> BaseArtifactUploader {
    let uploader = self.build().unwrap();

    if uploader.context_behavior == ContextBehavior::Init {
      init_uploader_stack(&uploader);
    } else if uploader.context_behavior == ContextBehavior::PushPop {
      push_uploader(&uploader);
    }

    uploader
  }
}

impl Clone for BaseArtifactUploader {
  fn clone(&self) -> Self {
    BaseArtifactUploaderBuilder::default()
      .client(self.client.clone())
      .data(self.data.clone())
      .context_behavior(ContextBehavior::Disabled)
      .init()
  }
}

impl Drop for BaseArtifactUploader {
  fn drop(&mut self) {
    if self.context_behavior == ContextBehavior::PushPop {
      pop_uploader(self);
    }
  }
}

pub(crate) fn artifact_group_uploader_data_from_request(
  request: &CreateArtifactRequest,
) -> ArtifactGroupUploaderData {
  let mut new_data = ArtifactGroupUploaderData::new();
  new_data.project_id = request.project_id.to_string();
  new_data.run_id = request.run_id.clone();
  new_data.id = request.artifact_id.clone();
  new_data.ancestor_group_ids = request.artifact_data().ancestor_group_ids.clone();
  new_data
}

fn base_child_group_request(
  parent_data: &ArtifactGroupUploaderData,
  metadata: ArtifactUserMetadata,
) -> CreateArtifactRequest {
  let mut request = CreateArtifactRequest::new();
  request.project_id = parent_data.project_id.clone();
  request.run_id = parent_data.run_id.clone();
  request.artifact_id = Some(new_artifact_id()).into();
  let group_data = request.mut_artifact_data();
  group_data.user_metadata = Some(metadata).into();
  parent_data
    .ancestor_group_ids
    .iter()
    .for_each(|id| group_data.ancestor_group_ids.push(id.clone()));
  parent_data.id.as_ref().map(|id| group_data.ancestor_group_ids.push(id.clone()));
  group_data.client_creation_time = Some(time_now()).into();
  request
}

impl BaseArtifactUploader {
  pub fn create_base_child_group_request(
    &self,
    metadata: &UserMetadataBuilder,
  ) -> CreateArtifactRequest {
    base_child_group_request(&self.data, metadata.proto.clone())
  }

  pub fn create_child_group(
    &self,
    request: CreateArtifactRequest,
    use_context: bool,
  ) -> BaseArtifactUploader {
    self.client.upload_artifact(&request, None);
    BaseArtifactUploaderBuilder::default()
      .client(self.client.clone())
      .data(artifact_group_uploader_data_from_request(&request))
      .context_behavior(if use_context {
        ContextBehavior::PushPop
      } else {
        ContextBehavior::Disabled
      })
      .init()
  }

  pub fn child_uploader(&self, metadata: &UserMetadataBuilder) -> GenericArtifactUploader {
    let mut request = self.create_base_child_group_request(metadata);
    request
      .mut_artifact_data()
      .artifact_type = ArtifactType::ARTIFACT_TYPE_GENERIC.into();
    GenericArtifactUploader {
      base: self.create_child_group(request, true),
    }
  }

  pub fn child_uploader_2d(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
    let mut request = self.create_base_child_group_request(metadata);
    request
      .mut_artifact_data()
      .artifact_type = ArtifactType::ARTIFACT_TYPE_2D_GROUP.into();
    ArtifactUploader2d {
      base: self.create_child_group(request, false),
    }
  }

  pub fn child_uploader_3d(
    &self,
    metadata: &UserMetadataBuilder,
    base_transform: Transform3,
  ) -> ArtifactUploader3d {
    let mut request = self.create_base_child_group_request(metadata);
    let artifact_data = request.mut_artifact_data();
    artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_3D_GROUP.into();
    artifact_data
      .mut_group_3d()
      .base_transform = Some(base_transform).into();
    ArtifactUploader3d {
      base: self.create_child_group(request, false),
    }
  }

  pub fn upload_raw(&self, metadata: &UserMetadataBuilder, data: StructuredData) -> String {
    self.upload_raw_bytes(metadata, data.write_to_bytes().unwrap().as_slice())
  }

  pub fn upload_raw_bytes(&self, metadata: &UserMetadataBuilder, data: &[u8]) -> String {
    let request = base_child_group_request(&self.data, metadata.proto.clone());
    self.client.upload_artifact_raw_bytes(&request, Some(data));
    bs58::encode(request.artifact_id.write_to_bytes().unwrap()).into_string()
  }

  pub fn id(&self) -> String {
    bs58::encode(self.data.id.write_to_bytes().unwrap()).into_string()
  }
}

