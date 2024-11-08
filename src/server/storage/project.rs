use observation_tools_common::project::ProjectId;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: ProjectId,
    pub data: ProjectData,
}

pub type ProjectRowOrError = Result<ProjectRow, anyhow::Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectData {
    pub name: String,
}
