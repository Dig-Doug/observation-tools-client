use crate::artifacts::PublicSeriesId;
use crate::artifacts::SeriesBuilder;
use crate::artifacts::SeriesPointBuilder;
use crate::artifacts::UserMetadataBuilder;
use crate::client::Client;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::groups::GenericArtifactUploader;
use crate::run_id::RunId;
use crate::task_handle::TaskHandle;
use crate::util::encode_id_proto;
use crate::util::new_artifact_id;
use crate::util::time_now;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::BaseArtifactUploaderTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use crate::PublicArtifactId;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesIdTaskHandle;
use derive_builder::Builder;
use observation_tools_common::proto::artifact_data;
use observation_tools_common::proto::artifact_update;
use observation_tools_common::proto::create_artifact_request;
use observation_tools_common::proto::public_global_id;
use observation_tools_common::proto::ArtifactData;
use observation_tools_common::proto::ArtifactGroupUploaderData;
use observation_tools_common::proto::ArtifactId;
use observation_tools_common::proto::ArtifactType;
use observation_tools_common::proto::ArtifactUpdate;
use observation_tools_common::proto::CanonicalArtifactId;
use observation_tools_common::proto::CreateArtifactRequest;
use observation_tools_common::proto::Group3d;
use observation_tools_common::proto::PublicGlobalId;
use observation_tools_common::proto::SeriesId;
use observation_tools_common::proto::StructuredData;
use observation_tools_common::proto::Transform3;
use prost::Message;

#[derive(Builder, Debug)]
pub(crate) struct BaseArtifactUploader {
    pub(crate) client: Client,
    data: ArtifactGroupUploaderData,
}

impl BaseArtifactUploaderBuilder {
    pub fn init(&mut self) -> BaseArtifactUploader {
        let uploader = self.build().unwrap();
        uploader
    }
}

impl Clone for BaseArtifactUploader {
    fn clone(&self) -> Self {
        BaseArtifactUploaderBuilder::default()
            .client(self.client.clone())
            .data(self.data.clone())
            .init()
    }
}

pub(crate) fn artifact_group_uploader_data_from_request(
    request: &CreateArtifactRequest,
) -> ArtifactGroupUploaderData {
    ArtifactGroupUploaderData {
        project_id: request.project_id.clone(),
        run_id: request.run_id.clone(),
        id: request.artifact_id.clone(),
        ancestor_group_ids: match request.data {
            Some(create_artifact_request::Data::ArtifactData(ref data)) => {
                data.ancestor_group_ids.clone()
            }
            _ => vec![],
        },
    }
}

impl BaseArtifactUploader {
    pub(crate) fn project_global_id(&self) -> PublicGlobalId {
        PublicGlobalId {
            data: Some(public_global_id::Data::ProjectId(
                self.data.project_id.clone().unwrap_or_default(),
            )),
        }
    }

    pub(crate) fn global_id(&self) -> PublicGlobalId {
        PublicGlobalId {
            data: Some(public_global_id::Data::CanonicalArtifactId(
                CanonicalArtifactId {
                    project_id: self.data.project_id.clone(),
                    artifact_id: match self.data.id.as_ref() {
                        Some(id) => Some(id.clone()),
                        None => None,
                    },
                },
            )),
        }
    }

    pub fn run_id(&self) -> RunId {
        RunId {
            id: encode_id_proto(&self.global_id()),
        }
    }

    fn base_artifact_request(
        &self,
        artifact_id: ArtifactId,
        series_point: Option<&SeriesPointBuilder>,
    ) -> CreateArtifactRequest {
        CreateArtifactRequest {
            project_id: self.data.project_id.clone(),
            run_id: self.data.run_id.clone(),
            artifact_id: Some(artifact_id.clone()),
            series_point: series_point.map(|b| b.proto.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn base_create_artifact_request<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPointBuilder>,
    ) -> CreateArtifactRequest {
        let mut request = self.base_artifact_request(new_artifact_id(), series_point);
        request.data = Some(create_artifact_request::Data::ArtifactData(ArtifactData {
            user_metadata: Some(metadata.into().proto.clone()),
            ancestor_group_ids: self
                .data
                .ancestor_group_ids
                .iter()
                .chain(self.data.id.as_ref())
                .cloned()
                .collect(),
            client_creation_time: Some(time_now()),
            ..Default::default()
        }));
        request
    }

    pub(crate) fn create_child_group(
        &self,
        request: CreateArtifactRequest,
    ) -> Result<BaseArtifactUploaderTaskHandle, ClientError> {
        Ok(self
            .client
            .upload_artifact(&request, None)?
            .map_handle(|_unused| {
                BaseArtifactUploaderBuilder::default()
                    .client(self.client.clone())
                    .data(artifact_group_uploader_data_from_request(&request))
                    .init()
            }))
    }

    pub fn child_uploader<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        request.data = Some(create_artifact_request::Data::ArtifactData(ArtifactData {
            artifact_type: ArtifactType::Generic.into(),
            ..Default::default()
        }));
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| GenericArtifactUploader { base: result }))
    }

    pub fn child_uploader_2d<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        request.data = Some(create_artifact_request::Data::ArtifactData(ArtifactData {
            artifact_type: ArtifactType::ArtifactType2dGroup.into(),
            ..Default::default()
        }));
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| ArtifactUploader2d { base: result }))
    }

    pub fn child_uploader_3d<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        base_transform: Transform3,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        request.data = Some(create_artifact_request::Data::ArtifactData(ArtifactData {
            artifact_type: ArtifactType::ArtifactType3dGroup.into(),
            type_data: Some(artifact_data::TypeData::Group3d(Group3d {
                base_transform: Some(base_transform),
                ..Default::default()
            })),
            ..Default::default()
        }));
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| ArtifactUploader3d { base: result }))
    }

    pub fn series<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, None);
        request.data = Some(create_artifact_request::Data::ArtifactData(ArtifactData {
            artifact_type: ArtifactType::Series.into(),
            type_data: Some(artifact_data::TypeData::Series(series.proto.clone())),
            ..Default::default()
        }));
        Ok(self
            .client
            .upload_artifact(&request, None)?
            .map_handle(|id| PublicSeriesId {
                proto: SeriesId {
                    artifact_id: Some(id.id),
                },
            }))
    }

    pub fn upload_raw<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        data: StructuredData,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.upload_raw_bytes(metadata, data.encode_to_vec().as_slice(), series_point)
    }

    pub fn upload_raw_bytes<M: Into<UserMetadataBuilder>>(
        &self,
        metadata: M,
        data: &[u8],
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let request = self.base_create_artifact_request(metadata, series_point);
        self.client.upload_artifact_raw_bytes(&request, Some(data))
    }

    pub fn update_raw(
        &self,
        artifact_id: &PublicArtifactId,
        data: StructuredData,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        self.update_raw_bytes(&artifact_id, data.encode_to_vec().as_slice(), series_point)
    }

    pub fn update_raw_bytes(
        &self,
        artifact_id: &PublicArtifactId,
        data: &[u8],
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let mut request = self.base_artifact_request(artifact_id.id.clone(), series_point);
        request.data = Some(create_artifact_request::Data::ArtifactUpdate(
            ArtifactUpdate {
                operation: artifact_update::Operation::Update.into(),
            },
        ));
        self.client.upload_artifact_raw_bytes(&request, Some(data))
    }

    pub fn id(&self) -> String {
        bs58::encode(self.data.id.clone().unwrap_or_default().encode_to_vec()).into_string()
    }
}
