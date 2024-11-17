use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::project::ProjectId;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ResourceType {
    Project = 1,
    Artifact = 2,
}

pub trait ResourceId: Debug + Clone + Hash + Sync + Send + Eq + 'static {
    fn resource_type() -> ResourceType;
}

impl ResourceId for ProjectId {
    fn resource_type() -> ResourceType {
        ResourceType::Project
    }
}

impl ResourceId for AbsoluteArtifactId {
    fn resource_type() -> ResourceType {
        ResourceType::Artifact
    }
}

/*
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Project(ProjectId),
    Artifact(AbsoluteArtifactId),
}

impl ResourceId {
    pub fn resource_type(&self) -> ResourceType {
        match self {
            ResourceId::Project(_) => ResourceType::Project,
            ResourceId::Artifact(_) => ResourceType::Artifact,
        }
    }
}


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

impl From<ResourceId> for GlobalId {
    fn from(value: ResourceId) -> Self {
        match value {
            ResourceId::Project(project_id) => project_id.into(),
            ResourceId::Artifact(artifact_id) => artifact_id.into(),
        }
    }
}
*/
