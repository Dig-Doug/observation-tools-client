use crate::artifacts::Transform3;
use crate::artifacts::*;
use crate::math::Graph;
use crate::project::ProjectId;
use crate::GlobalId;
use anyhow::anyhow;
use async_graphql::ID;
use chrono::DateTime;
use chrono::Utc;
use clap::Args;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Args)]
pub struct ArtifactId {
    #[arg(id = "artifact-uuid", short = 'a', long = "artifact-id")]
    pub uuid: Uuid,
}

impl ArtifactId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Args)]
pub struct AbsoluteArtifactId {
    #[command(flatten)]
    pub project_id: ProjectId,
    #[command(flatten)]
    pub artifact_id: ArtifactId,
    #[arg(skip)]
    pub series_context: SeriesContext,
}

impl From<AbsoluteArtifactId> for GlobalId {
    fn from(value: AbsoluteArtifactId) -> Self {
        GlobalId::Artifact(value)
    }
}

impl From<AbsoluteArtifactId> for ID {
    fn from(id: AbsoluteArtifactId) -> Self {
        let global_id: GlobalId = id.into();
        global_id.into()
    }
}

impl TryFrom<ID> for AbsoluteArtifactId {
    type Error = anyhow::Error;

    fn try_from(id: ID) -> Result<Self, Self::Error> {
        match id.try_into()? {
            GlobalId::Artifact(artifact_id) => Ok(artifact_id),
            _ => Err(anyhow!("Not an AbsoluteArtifactId")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SeriesContext {
    None,
    Filter { series_point: Option<SeriesPoint> },
}

impl Default for SeriesContext {
    fn default() -> Self {
        SeriesContext::None
    }
}

impl SeriesContext {
    pub fn series_point(&self) -> Option<SeriesPoint> {
        match self {
            SeriesContext::None => None,
            SeriesContext::Filter { series_point } => series_point.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Args)]
pub struct ArtifactVersionId {
    #[arg(id = "version-uuid", short = 'v', long = "version-id")]
    pub uuid: Uuid,
}

impl ArtifactVersionId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Args)]
pub struct AbsoluteArtifactVersionId {
    #[command(flatten)]
    pub project_id: ProjectId,
    #[command(flatten)]
    pub artifact_id: ArtifactId,
    #[command(flatten)]
    pub version_id: ArtifactVersionId,
}

impl From<AbsoluteArtifactVersionId> for GlobalId {
    fn from(value: AbsoluteArtifactVersionId) -> Self {
        GlobalId::ArtifactVersion(value)
    }
}

impl From<AbsoluteArtifactVersionId> for ID {
    fn from(id: AbsoluteArtifactVersionId) -> Self {
        let global_id: GlobalId = id.into();
        global_id.into()
    }
}

impl TryFrom<ID> for AbsoluteArtifactVersionId {
    type Error = anyhow::Error;

    fn try_from(id: ID) -> Result<Self, Self::Error> {
        match id.try_into()? {
            GlobalId::ArtifactVersion(artifact_id) => Ok(artifact_id),
            _ => Err(anyhow!("Not an AbsoluteArtifactVersionId")),
        }
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
