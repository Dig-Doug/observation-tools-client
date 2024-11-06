mod artifact;
pub mod sqlite;

pub use artifact::ArtifactStorage;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifacts::SeriesPoint;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use uuid::Uuid;

pub struct ArtifactVersion {
    pub project_id: ProjectId,
    pub run_id: Option<RunId>,
    pub artifact_id: ArtifactId,
    pub version_id: Uuid,
    pub version_data: ArtifactData,
    pub series_point: Option<SeriesPoint>,
}
