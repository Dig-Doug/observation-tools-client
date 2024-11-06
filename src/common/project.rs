use crate::GlobalId;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ProjectId {
    pub id: Uuid,
}

impl From<ProjectId> for GlobalId {
    fn from(project_id: ProjectId) -> Self {
        GlobalId::Project(project_id)
    }
}
