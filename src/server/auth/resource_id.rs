use crate::auth::permission::IntoResourceId;
use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use observation_tools_common::project::ProjectId;
use observation_tools_common::GlobalId;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Project,
    Artifact,
    ArtifactVersion,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Project(ProjectId),
    Artifact(AbsoluteArtifactId),
    ArtifactVersion(AbsoluteArtifactVersionId),
}

impl ResourceId {
    pub fn resource_type(&self) -> ResourceType {
        match self {
            ResourceId::Project(_) => ResourceType::Project,
            ResourceId::Artifact(_) => ResourceType::Artifact,
            ResourceId::ArtifactVersion(_) => ResourceType::ArtifactVersion,
        }
    }
}

impl IntoResourceId for ProjectId {}
impl IntoResourceId for AbsoluteArtifactId {}
impl IntoResourceId for AbsoluteArtifactVersionId {}

impl From<ProjectId> for ResourceId {
    fn from(value: ProjectId) -> Self {
        ResourceId::Project(value)
    }
}

impl From<AbsoluteArtifactId> for ResourceId {
    fn from(value: AbsoluteArtifactId) -> Self {
        ResourceId::Artifact(value)
    }
}

impl From<AbsoluteArtifactVersionId> for ResourceId {
    fn from(value: AbsoluteArtifactVersionId) -> Self {
        ResourceId::ArtifactVersion(value)
    }
}

impl From<ResourceId> for GlobalId {
    fn from(value: ResourceId) -> Self {
        match value {
            ResourceId::Project(project_id) => project_id.into(),
            ResourceId::Artifact(artifact_id) => artifact_id.into(),
            ResourceId::ArtifactVersion(artifact_version_id) => artifact_version_id.into(),
        }
    }
}
