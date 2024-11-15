use crate::storage::project::ProjectRow;
use crate::storage::project::ProjectRowOrError;
use crate::storage::sqlite::SqliteArtifactStorage;
use crate::storage::ArtifactVersionRow;
use crate::storage::ArtifactVersionRowOrError;
use futures_util::TryStream;
use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use observation_tools_common::artifact::StructuredData;
use observation_tools_common::project::ProjectId;
use std::collections::HashMap;
use std::error::Error;
use tokio_util::bytes::Bytes;

// Note: It would be nice to use a trait and have separate implementations, but
// it's not easy with the `impl TryStream<>`
#[derive(Clone)]
pub enum ArtifactStorage {
    Local(SqliteArtifactStorage),
}

impl ArtifactStorage {
    pub async fn create_project(&self, project_row: ProjectRow) -> Result<(), anyhow::Error> {
        match self {
            ArtifactStorage::Local(storage) => storage.create_project(project_row).await,
        }
    }

    pub async fn read_projects(
        &self,
        projects: &Vec<ProjectId>,
    ) -> Result<HashMap<ProjectId, ProjectRowOrError>, anyhow::Error> {
        match self {
            ArtifactStorage::Local(storage) => storage.read_projects(projects).await,
        }
    }

    pub async fn read_artifact_versions(
        &self,
        versions: &Vec<AbsoluteArtifactVersionId>,
    ) -> Result<HashMap<AbsoluteArtifactVersionId, ArtifactVersionRowOrError>, anyhow::Error> {
        match self {
            ArtifactStorage::Local(storage) => storage.read_artifact_versions(versions).await,
        }
    }

    pub async fn read_versions_for_artifact(
        &self,
        artifact_id: &AbsoluteArtifactId,
    ) -> Result<Vec<ArtifactVersionRowOrError>, anyhow::Error> {
        todo!("Impl")
    }

    pub async fn write_artifact_version<E: Error + Send + Sync + 'static>(
        &self,
        version: ArtifactVersionRow,
        bytes_stream: Option<impl TryStream<Ok = Bytes, Error = E> + Unpin>,
    ) -> Result<(), anyhow::Error> {
        match self {
            ArtifactStorage::Local(storage) => {
                storage.write_artifact_version(version, bytes_stream).await
            }
        }
    }

    pub async fn read_artifact_version_payload(
        &self,
        version_id: &AbsoluteArtifactVersionId,
    ) -> Result<Option<StructuredData>, anyhow::Error> {
        match self {
            ArtifactStorage::Local(storage) => {
                storage.read_artifact_version_payload(version_id).await
            }
        }
    }
}
