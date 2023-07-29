use crate::builders::UserMetadataBuilder;
use crate::builders::{PublicSeriesId, SeriesBuilder, SeriesPointBuilder};
use crate::client::Client;
use crate::run_id::RunId;
use crate::util::time_now;
use crate::util::ClientError;
use crate::util::{encode_id_proto, new_artifact_id};
use crate::PublicArtifactId;

use artifacts_api_rust_proto::ArtifactId;
use artifacts_api_rust_proto::CreateArtifactRequest;
use artifacts_api_rust_proto::Transform3;
use artifacts_api_rust_proto::{artifact_update, ArtifactType};
use artifacts_api_rust_proto::{ArtifactGroupUploaderData, SeriesId};
use artifacts_api_rust_proto::{PublicGlobalId, StructuredData};
use derive_builder::Builder;

use crate::uploaders::{ArtifactUploader2d, ArtifactUploader3d, GenericArtifactUploader};
use protobuf::Message;

#[derive(Builder)]
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
    let mut new_data = ArtifactGroupUploaderData::new();
    new_data.project_id = request.project_id.clone();
    new_data.run_id = request.run_id.clone();
    new_data.id = request.artifact_id.clone();
    new_data.ancestor_group_ids = request.artifact_data().ancestor_group_ids.clone();
    new_data
}

impl BaseArtifactUploader {
    pub(crate) fn project_global_id(&self) -> PublicGlobalId {
        let mut proto = PublicGlobalId::new();
        *proto.mut_project_id() = self.data.project_id.clone().unwrap_or_default();
        proto
    }

    pub(crate) fn global_id(&self) -> PublicGlobalId {
        let mut proto = PublicGlobalId::new();
        let id = proto.mut_canonical_artifact_id();
        id.project_id = self.data.project_id.clone();
        id.artifact_id = self.data.run_id.id.clone();
        proto
    }

    pub fn run_id(&self) -> RunId {
        RunId {
            id: encode_id_proto(&self.global_id()),
        }
    }

    pub fn base_artifact_request(
        &self,
        artifact_id: ArtifactId,
        series_point: Option<&SeriesPointBuilder>,
    ) -> CreateArtifactRequest {
        let mut request = CreateArtifactRequest::new();
        request.project_id = self.data.project_id.clone();
        request.run_id = self.data.run_id.clone();
        request.artifact_id = Some(artifact_id.clone()).into();
        request.series_point = series_point.map(|b| b.proto.clone()).into();
        request
    }

    pub fn base_create_artifact_request(
        &self,
        metadata: &UserMetadataBuilder,
        series_point: Option<&SeriesPointBuilder>,
    ) -> CreateArtifactRequest {
        let mut request = self.base_artifact_request(new_artifact_id(), series_point);
        let group_data = request.mut_artifact_data();
        group_data.user_metadata = Some(metadata.proto.clone()).into();
        self.data
            .ancestor_group_ids
            .iter()
            .for_each(|id| group_data.ancestor_group_ids.push(id.clone()));
        self.data
            .id
            .as_ref()
            .map(|id| group_data.ancestor_group_ids.push(id.clone()));
        group_data.client_creation_time = Some(time_now()).into();
        request
    }

    pub async fn create_child_group(
        &self,
        request: CreateArtifactRequest,
    ) -> Result<BaseArtifactUploader, ClientError> {
        self.client.upload_artifact(&request, None).await?;
        Ok(BaseArtifactUploaderBuilder::default()
            .client(self.client.clone())
            .data(artifact_group_uploader_data_from_request(&request))
            .init())
    }

    pub async fn child_uploader(
        &self,
        metadata: &UserMetadataBuilder,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<GenericArtifactUploader, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_GENERIC.into();
        Ok(GenericArtifactUploader {
            base: self.create_child_group(request).await?,
        })
    }

    pub async fn child_uploader_2d(
        &self,
        metadata: &UserMetadataBuilder,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<ArtifactUploader2d, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_2D_GROUP.into();
        Ok(ArtifactUploader2d {
            base: self.create_child_group(request).await?,
        })
    }

    pub async fn child_uploader_3d(
        &self,
        metadata: &UserMetadataBuilder,
        base_transform: Transform3,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<ArtifactUploader3d, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, series_point);
        let artifact_data = request.mut_artifact_data();
        artifact_data.artifact_type = ArtifactType::ARTIFACT_TYPE_3D_GROUP.into();
        artifact_data.mut_group_3d().base_transform = Some(base_transform).into();
        Ok(ArtifactUploader3d {
            base: self.create_child_group(request).await?,
        })
    }

    pub async fn series(
        &self,
        metadata: &UserMetadataBuilder,
        series: &SeriesBuilder,
    ) -> Result<PublicSeriesId, ClientError> {
        let mut request = self.base_create_artifact_request(metadata, None);
        request.mut_artifact_data().artifact_type = ArtifactType::ARTIFACT_TYPE_SERIES.into();
        *request.mut_artifact_data().mut_series() = series.proto.clone();
        let id = self.client.upload_artifact(&request, None).await?;
        let mut series_id_proto = SeriesId::new();
        series_id_proto.artifact_id = Some(id.id).into();
        Ok(PublicSeriesId {
            proto: series_id_proto,
        })
    }

    pub async fn upload_raw(
        &self,
        metadata: &UserMetadataBuilder,
        data: StructuredData,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactId, ClientError> {
        self.upload_raw_bytes(
            metadata,
            data.write_to_bytes().unwrap().as_slice(),
            series_point,
        )
        .await
    }

    pub async fn upload_raw_bytes(
        &self,
        metadata: &UserMetadataBuilder,
        data: &[u8],
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactId, ClientError> {
        let request = self.base_create_artifact_request(metadata, series_point);
        self.client
            .upload_artifact_raw_bytes(&request, Some(data))
            .await
    }

    pub async fn update_raw(
        &self,
        artifact_id: &PublicArtifactId,
        data: StructuredData,
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactId, ClientError> {
        self.update_raw_bytes(
            &artifact_id,
            data.write_to_bytes().unwrap().as_slice(),
            series_point,
        )
        .await
    }

    pub async fn update_raw_bytes(
        &self,
        artifact_id: &PublicArtifactId,
        data: &[u8],
        series_point: Option<&SeriesPointBuilder>,
    ) -> Result<PublicArtifactId, ClientError> {
        let mut request = self.base_artifact_request(artifact_id.id.clone(), series_point);
        let artifact_update = request.mut_artifact_update();
        artifact_update.operation = artifact_update::Operation::OPERATION_UPDATE.into();
        self.client
            .upload_artifact_raw_bytes(&request, Some(data))
            .await
    }

    pub fn id(&self) -> String {
        bs58::encode(self.data.id.write_to_bytes().unwrap()).into_string()
    }
}
