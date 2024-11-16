use crate::GlobalId;
use async_graphql::ID;
use clap::Args;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Args)]
pub struct ProjectId {
    #[arg(id = "project-uuid", short = 'p', long = "project-id")]
    pub id: Uuid,
}

impl ProjectId {
    pub fn new() -> Self {
        ProjectId { id: Uuid::new_v4() }
    }
}

impl From<ProjectId> for GlobalId {
    fn from(project_id: ProjectId) -> Self {
        GlobalId::Project(project_id)
    }
}

impl From<ProjectId> for ID {
    fn from(project_id: ProjectId) -> Self {
        let global_id: GlobalId = project_id.into();
        global_id.into()
    }
}
