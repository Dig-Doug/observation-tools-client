use crate::artifacts::Series;
use crate::artifacts::SeriesPoint;
use crate::artifacts::UserMetadata;
use crate::client::Client;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::groups::GenericArtifactUploader;
use crate::task_handle::TaskHandle;
use crate::util::encode_id_proto;
use crate::util::time_now;
use crate::util::ClientError;
use crate::ArtifactUploader2dTaskHandle;
use crate::ArtifactUploader3dTaskHandle;
use crate::BaseArtifactUploaderTaskHandle;
use crate::GenericArtifactUploaderTaskHandle;
use crate::PublicArtifactId;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesId;
use crate::PublicSeriesIdTaskHandle;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifact::ArtifactType;
use observation_tools_common::artifact::Group3d;
use observation_tools_common::artifact::StructuredData;
use observation_tools_common::artifacts::SeriesId;
use observation_tools_common::artifacts::Transform3;
use observation_tools_common::create_artifact::CreateArtifactRequest;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use prost::Message;
use std::iter::once;

#[derive(Debug, Clone)]
pub(crate) struct BaseArtifactUploader {
    pub(crate) client: Client,
    pub(crate) project_id: ProjectId,
    pub(crate) run_id: RunId,
    pub(crate) id: ArtifactId,
    pub(crate) ancestor_group_ids: Vec<ArtifactId>,
}

impl BaseArtifactUploader {
    pub(crate) fn base_create_artifact_request<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        type_data: ArtifactType,
        series_point: Option<&SeriesPoint>,
    ) -> CreateArtifactRequest {
        CreateArtifactRequest {
            project_id: self.project_id.clone(),
            run_id: Some(self.run_id.clone()),
            artifact_id: ArtifactId::new(),
            payload: ArtifactData {
                user_metadata: metadata.into(),
                ancestor_group_ids: self
                    .ancestor_group_ids
                    .iter()
                    .chain(once(&self.id))
                    .cloned()
                    .collect(),
                client_creation_time: time_now(),
                artifact_type: type_data,
            },
            series_point: series_point.cloned(),
        }
    }

    pub(crate) fn create_child_group(
        &self,
        request: CreateArtifactRequest,
    ) -> Result<BaseArtifactUploaderTaskHandle, ClientError> {
        Ok(self
            .client
            .upload_artifact(request, None)?
            .map_handle(|child_id| {
                let mut child = self.clone();
                child.ancestor_group_ids.push(self.id.clone());
                child.id = child_id.id;
                child
            }))
    }

    pub fn child_uploader<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPoint>,
    ) -> Result<GenericArtifactUploaderTaskHandle, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Generic, series_point);
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| GenericArtifactUploader { base: result }))
    }

    pub fn child_uploader_2d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPoint>,
    ) -> Result<ArtifactUploader2dTaskHandle, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Group2D, series_point);
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| ArtifactUploader2d { base: result }))
    }

    pub fn child_uploader_3d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        base_transform: Transform3,
        series_point: Option<&SeriesPoint>,
    ) -> Result<ArtifactUploader3dTaskHandle, ClientError> {
        let request = self.base_create_artifact_request(
            metadata,
            ArtifactType::Group3d(Group3d { base_transform }),
            series_point,
        );
        Ok(self
            .create_child_group(request)?
            .map_handle(|result| ArtifactUploader3d { base: result }))
    }

    pub fn series<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series: Series,
    ) -> Result<PublicSeriesIdTaskHandle, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Series(series), None);
        Ok(self
            .client
            .upload_artifact(request, None)?
            .map_handle(|id| PublicSeriesId {
                id: SeriesId { artifact_id: id.id },
            }))
    }

    pub(crate) fn upload_raw<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        data: StructuredData,
        series_point: Option<&SeriesPoint>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let bytes = rmp_serde::to_vec(&data).map_err(ClientError::from_string)?;
        self.upload_raw_bytes(metadata, &bytes, series_point)
    }

    pub(crate) fn upload_raw_bytes<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        data: &[u8],
        series_point: Option<&SeriesPoint>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Artifact, series_point);
        self.client.upload_artifact_raw_bytes(request, Some(data))
    }

    pub(crate) fn update_raw(
        &self,
        artifact_id: ArtifactId,
        data: StructuredData,
        series_point: Option<&SeriesPoint>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let bytes = rmp_serde::to_vec(&data).map_err(ClientError::from_string)?;
        self.update_raw_bytes(artifact_id, &bytes, series_point)
    }

    pub(crate) fn update_raw_bytes(
        &self,
        artifact_id: ArtifactId,
        data: &[u8],
        series_point: Option<&SeriesPoint>,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        todo!("impl");
        /*
        let mut request = self.base_artifact_request(artifact_id, series_point);
        request.data = Some(create_artifact_request::Data::ArtifactUpdate(
            ArtifactUpdate {
                operation: artifact_update::Operation::Update.into(),
            },
        ));
        self.client.upload_artifact_raw_bytes(&request, Some(data))
         */
    }
}
