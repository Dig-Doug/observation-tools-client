pub mod artifact;
pub mod project;
pub mod sqlite;
mod util;

use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifacts::SeriesPoint;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use observation_tools_common::GlobalId;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactVersionRow {
    pub project_id: ProjectId,
    pub run_id: Option<RunId>,
    pub artifact_id: ArtifactId,
    pub version_id: Uuid,
    pub version_data: ArtifactData,
    pub series_point: Option<SeriesPoint>,
}

impl ArtifactVersionRow {
    pub fn absolute_id(&self) -> AbsoluteArtifactId {
        AbsoluteArtifactId {
            project_id: self.project_id.clone(),
            artifact_id: self.artifact_id.clone(),
        }
    }

    pub fn global_id(&self) -> GlobalId {
        self.absolute_id().into()
    }
}
