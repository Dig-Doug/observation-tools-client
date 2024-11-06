extern crate alloc;
extern crate core;

use crate::project::ProjectId;
use anyhow::anyhow;
use uuid::Uuid;

pub mod artifact;
pub mod artifacts;
pub mod create_artifact;
pub mod math;
pub mod project;
pub mod run;

pub enum GlobalId {
    Project(ProjectId),
}

const GLOBAL_ID_PREFIX_PROJECT: &str = "p";

impl TryFrom<GlobalId> for String {
    type Error = anyhow::Error;

    fn try_from(value: GlobalId) -> Result<Self, Self::Error> {
        let prefix = match value {
            GlobalId::Project(_) => GLOBAL_ID_PREFIX_PROJECT,
        };
        let suffix = match value {
            GlobalId::Project(id) => bs58::encode(id.id.into_bytes()).into_string(),
        };
        Ok(format!("{}_{}", prefix, suffix))
    }
}

impl TryFrom<String> for GlobalId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('_').collect();
        let prefix = parts.get(0).ok_or(anyhow!("ID has no prefix: {}", value))?;
        let suffix = parts.get(1).ok_or(anyhow!("ID has no suffix: {}", value))?;
        match *prefix {
            GLOBAL_ID_PREFIX_PROJECT => {
                let id = bs58::decode(suffix).into_vec()?;
                let id = ProjectId {
                    id: Uuid::from_slice(&id)?,
                };
                Ok(GlobalId::Project(id))
            }
            _ => Err(anyhow!("Unknown prefix: {}", prefix)),
        }
    }
}
