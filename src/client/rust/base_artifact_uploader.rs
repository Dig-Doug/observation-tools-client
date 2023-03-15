use crate::api::new_artifact_id;
use crate::artifact_uploader_2d::ArtifactUploader2d;
use crate::artifact_uploader_3d::ArtifactUploader3d;
use crate::client::Client;
use crate::generic_artifact_uploader::GenericArtifactUploader;
use crate::run_id::RunId;
use crate::uploader_stack::{init_uploader_stack, pop_uploader, push_uploader};
use crate::user_metadata::UserMetadataBuilder;
use crate::util::{encode_id_proto, time_now, ClientError};
use crate::PublicArtifactId;
use artifacts_api_rust_proto::{
    ArtifactGroupUploaderData, ArtifactType, ArtifactUserMetadata, CreateArtifactRequest,
    StructuredData, Transform3,
};
use derive_builder::Builder;
use protobuf::Message;

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
    parent_data
        .id
        .as_ref()
        .map(|id| group_data.ancestor_group_ids.push(id.clone()));
    group_data.client_creation_time = Some(time_now()).into();
    request
}

impl BaseArtifactUploader {
    pub fn run_id(&self) -> RunId {
        RunId {
            id: encode_id_proto(&self.data.run_id.clone().unwrap_or_default()),
        }
    }

    pub fn create_base_child_group_request(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> CreateArtifactRequest {
        base_child_group_request(&self.data, metadata.proto.clone())
    }

    pub async fn create_child_group_async(
        &self,
        request: CreateArtifactRequest,
        use_context: bool,
    ) -> Result<BaseArtifactUploader, ClientError> {
        self.client.upload_artifact(&request, None).await?;
        Ok(BaseArtifactUploaderBuilder::default()
            .client(self.client.clone())
            .data(artifact_group_uploader_data_from_request(&request))
            .context_behavior(if use_context {
                ContextBehavior::PushPop
            } else {
                ContextBehavior::Disabled
            })
            .init())
    }

    pub fn create_child_group_old(
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
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_GENERIC.into();
        GenericArtifactUploader {
            base: self.create_child_group_old(request, true),
        }
    }

    pub async fn child_uploader_async(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<GenericArtifactUploader, ClientError> {
        let mut request = self.create_base_child_group_request(metadata);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_GENERIC.into();
        Ok(GenericArtifactUploader {
            base: self.create_child_group_async(request, true).await?,
        })
    }

    pub fn child_uploader_2d_old(&self, metadata: &UserMetadataBuilder) -> ArtifactUploader2d {
        let mut request = self.create_base_child_group_request(metadata);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_2D_GROUP.into();
        ArtifactUploader2d {
            base: self.create_child_group_old(request, false),
        }
    }

    pub async fn child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
    ) -> Result<ArtifactUploader2d, ClientError> {
        let mut request = self.create_base_child_group_request(metadata);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_2D_GROUP.into();
        Ok(ArtifactUploader2d {
            base: self.create_child_group_async(request, false).await?,
        })
    }

    pub fn child_uploader_3d_old(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3,
    ) -> ArtifactUploader3d {
        let mut request = self.create_base_child_group_request(metadata);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_3D_GROUP.into();
        artifact_data.mut_group_3d().base_transform = Some(base_transform).into();
        ArtifactUploader3d {
            base: self.create_child_group_old(request, false),
        }
    }

    pub async fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3,
    ) -> Result<ArtifactUploader3d, ClientError> {
        let mut request = self.create_base_child_group_request(metadata);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_3D_GROUP.into();
        artifact_data.mut_group_3d().base_transform = Some(base_transform).into();
        Ok(ArtifactUploader3d {
            base: self.create_child_group_async(request, false).await?,
        })
    }

    pub async fn upload_raw(
        &self,
        metadata: &UserMetadataBuilder,
        data: StructuredData,
    ) -> Result<PublicArtifactId, ClientError> {
        self.upload_raw_bytes(metadata, data.write_to_bytes().unwrap().as_slice())
            .await
    }

    pub async fn upload_raw_bytes(
        &self,
        metadata: &UserMetadataBuilder,
        data: &[u8],
    ) -> Result<PublicArtifactId, ClientError> {
        let request = base_child_group_request(&self.data, metadata.proto.clone());
        self.client
            .upload_artifact_raw_bytes(&request, Some(data))
            .await
    }

    pub fn id(&self) -> String {
        bs58::encode(self.data.id.write_to_bytes().unwrap()).into_string()
    }
}
