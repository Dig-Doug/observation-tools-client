use crate::artifacts::Transform3;
use crate::artifacts::*;
use crate::math::Graph;
use crate::project::ProjectId;
use crate::GlobalId;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactId {
    pub uuid: Uuid,
}

impl ArtifactId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbsoluteArtifactId {
    pub project_id: ProjectId,
    pub artifact_id: ArtifactId,
}

impl From<AbsoluteArtifactId> for GlobalId {
    fn from(value: AbsoluteArtifactId) -> Self {
        GlobalId::Artifact(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactData {
    pub ancestor_group_ids: Vec<ArtifactId>,
    pub user_metadata: UserMetadata,
    pub artifact_type: ArtifactType,
    pub client_creation_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArtifactType {
    Artifact,
    Generic,
    Group2D,
    Group3d(Group3d),
    Group2dIn3d(Map2dTo3dData),
    RunStage(RunStageData),
    RootGroup,
    Series(Series),
}

impl ArtifactType {
    pub fn as_string(&self) -> String {
        match self {
            ArtifactType::Artifact => "Artifact",
            ArtifactType::Generic => "Generic",
            ArtifactType::Group2D => "Group2D",
            ArtifactType::Group3d(_) => "Group3d",
            ArtifactType::Group2dIn3d(_) => "Group2dIn3d",
            ArtifactType::RunStage(_) => "RunStage",
            ArtifactType::RootGroup => "RootGroup",
            ArtifactType::Series(_) => "Series",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunStageData {
    pub previous_run_stage_ids: Vec<ArtifactId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map2dTo3dData {
    pub to_3d_transform: Transform3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group3d {
    pub base_transform: Transform3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StructuredData {
    Graph(Graph),
    Image2(Image2),
    Object2(Object2),
    Object3(Object3),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArtifactUpdate {
    Create,
    Update,
}
