use crate::artifacts::Series;
use crate::artifacts::SeriesPoint;
use crate::artifacts::UserMetadata;
use crate::client::Client;
use crate::groups::ArtifactUploader2d;
use crate::groups::ArtifactUploader3d;
use crate::groups::GenericArtifactUploader;
use crate::groups::Object2Updater;
use crate::task_handle::Object2UpdaterTaskHandle;
use crate::task_handle::TaskHandle;
use crate::util::time_now;
use crate::util::ClientError;
use crate::PublicArtifactIdTaskHandle;
use crate::PublicSeriesId;
use crate::PublicSeriesIdTaskHandle;
use anyhow::Context;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifact::ArtifactType;
use observation_tools_common::artifact::Group3d;
use observation_tools_common::artifact::StructuredData;
use observation_tools_common::artifacts::Object1;
use observation_tools_common::artifacts::Object2;
use observation_tools_common::artifacts::Object3;
use observation_tools_common::artifacts::SeriesId;
use observation_tools_common::artifacts::Transform2;
use observation_tools_common::artifacts::Transform3;
use observation_tools_common::create_artifact::CreateArtifactRequest;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use std::any::TypeId;
use std::iter::once;

#[derive(Debug, Clone)]
pub(crate) struct BaseArtifactUploader {
    pub(crate) client: Client,
    pub(crate) project_id: ProjectId,
    pub(crate) run_id: RunId,
    pub(crate) id: ArtifactId,
    pub(crate) ancestor_group_ids: Vec<ArtifactId>,
    pub(crate) handle: PublicArtifactIdTaskHandle,
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
    ) -> Result<BaseArtifactUploader, ClientError> {
        let artifact_handle = self.client.upload_artifact(request, None)?;
        Ok(BaseArtifactUploader {
            client: self.client.clone(),
            project_id: self.project_id.clone(),
            run_id: self.run_id.clone(),
            id: artifact_handle.id.clone(),
            ancestor_group_ids: self
                .ancestor_group_ids
                .iter()
                .chain(once(&self.id))
                .cloned()
                .collect(),
            handle: artifact_handle,
        })
    }

    pub fn child_uploader<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPoint>,
    ) -> Result<GenericArtifactUploader, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Generic, series_point);
        Ok(GenericArtifactUploader {
            base: self.create_child_group(request)?,
        })
    }

    pub fn child_uploader_2d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        series_point: Option<&SeriesPoint>,
    ) -> Result<ArtifactUploader2d, ClientError> {
        let request =
            self.base_create_artifact_request(metadata, ArtifactType::Group2D, series_point);
        Ok(ArtifactUploader2d {
            base: self.create_child_group(request)?,
        })
    }

    pub fn child_uploader_3d<M: Into<UserMetadata>>(
        &self,
        metadata: M,
        base_transform: Transform3,
        series_point: Option<&SeriesPoint>,
    ) -> Result<ArtifactUploader3d, ClientError> {
        let request = self.base_create_artifact_request(
            metadata,
            ArtifactType::Group3d(Group3d { base_transform }),
            series_point,
        );
        Ok(ArtifactUploader3d {
            base: self.create_child_group(request)?,
        })
    }

    pub fn create_object1<M: Into<UserMetadata>, D: Into<Object1> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<PublicArtifactIdTaskHandle, ClientError> {
        let mut object1 = data.into();
        self.upload_raw(metadata, object1.try_into()?, None)
    }

    pub fn create_object2<M: Into<UserMetadata>, D: Into<Object2> + 'static>(
        &self,
        metadata: M,
        data: D,
    ) -> Result<Object2UpdaterTaskHandle, ClientError> {
        let metadata = metadata.into();
        let mut data = data.into();
        if TypeId::of::<D>() != TypeId::of::<Object2>() {
            // #implicit-transform
            data.add_transform(Transform2::identity());
        }

        Ok(self
            .upload_raw(
                metadata.clone(),
                data.clone()
                    .try_into()
                    .with_context(|| format!("Failed to parse object `{}`", metadata.name))?,
                data.series_point.as_ref(),
            )?
            .map_handle(|id| Object2Updater { id: id.id }))
    }

    pub fn update_object2<D: Into<Object2>>(
        &self,
        artifact: &Object2Updater,
        data: D,
    ) -> Result<(), ClientError> {
        let data = data.into();
        let series_point = data.series_point.clone();
        self.update_raw(artifact.id.clone(), data.try_into()?, series_point.as_ref())?;
        Ok(())
    }

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
        self.upload_raw(metadata, object3.try_into()?, None)
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

#[macro_export]
macro_rules! child_uploader_impl {
    () => {
        pub fn child_uploader<M: Into<UserMetadata>>(
            &self,
            metadata: M,
        ) -> Result<GenericArtifactUploader, ClientError> {
            self.base.child_uploader(metadata, None)
        }
    };
}

#[macro_export]
macro_rules! child_uploader_2d_impl {
    () => {
        pub fn child_uploader_2d<M: Into<UserMetadata>>(
            &self,
            metadata: M,
        ) -> Result<ArtifactUploader2d, ClientError> {
            self.base.child_uploader_2d(metadata, None)
        }
    };
}

#[macro_export]
macro_rules! child_uploader_3d_impl {
    () => {
        pub fn child_uploader_3d<M: Into<UserMetadata>>(
            &self,
            metadata: M,
            base_transform: Transform3,
        ) -> Result<ArtifactUploader3d, ClientError> {
            self.base.child_uploader_3d(metadata, base_transform, None)
        }
    };
}
