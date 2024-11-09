extern crate alloc;
extern crate core;

use crate::artifact::AbsoluteArtifactId;
use crate::project::ProjectId;
use anyhow::anyhow;
use async_graphql::ID;
use core::fmt::Display;
use core::fmt::Formatter;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::io::Cursor;

pub mod artifact;
pub mod artifacts;
pub mod create_artifact;
pub mod math;
pub mod project;
pub mod run;

#[derive(Debug, Clone)]
pub enum GlobalId {
    Project(ProjectId),
    Artifact(AbsoluteArtifactId),
}

const GLOBAL_ID_PREFIX_PROJECT: &str = "p";
const GLOBAL_ID_PREFIX_ARTIFACT: &str = "a";

impl From<GlobalId> for String {
    fn from(value: GlobalId) -> Self {
        let prefix = match value {
            GlobalId::Project(_) => GLOBAL_ID_PREFIX_PROJECT,
            GlobalId::Artifact(_) => GLOBAL_ID_PREFIX_ARTIFACT,
        };
        let suffix = match value {
            GlobalId::Project(id) => rmp_with_bs58_encode(id),
            GlobalId::Artifact(id) => rmp_with_bs58_encode(id),
        };
        format!("{}_{}", prefix, suffix)
    }
}

fn rmp_with_bs58_encode<T: Serialize>(value: T) -> String {
    let bytes = rmp_serde::to_vec(&value).expect("Failed to serialize id");
    bs58::encode(bytes).into_string()
}

impl TryFrom<String> for GlobalId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('_').collect();
        let prefix = parts.get(0).ok_or(anyhow!("ID has no prefix: {}", value))?;
        let suffix = parts.get(1).ok_or(anyhow!("ID has no suffix: {}", value))?;
        match *prefix {
            GLOBAL_ID_PREFIX_PROJECT => Ok(GlobalId::Project(bs58_with_rmp_decode(&suffix)?)),
            GLOBAL_ID_PREFIX_ARTIFACT => Ok(GlobalId::Artifact(bs58_with_rmp_decode(&suffix)?)),
            _ => Err(anyhow!("Unknown prefix: {}", prefix)),
        }
    }
}

fn bs58_with_rmp_decode<T: for<'a> Deserialize<'a>>(value: &&str) -> Result<T, anyhow::Error> {
    let bytes = bs58::decode(value).into_vec()?;
    Ok(rmp_serde::from_read(&mut Cursor::new(bytes))?)
}

impl From<GlobalId> for ID {
    fn from(value: GlobalId) -> Self {
        ID(String::from(value))
    }
}

impl TryFrom<ID> for GlobalId {
    type Error = anyhow::Error;

    fn try_from(value: ID) -> Result<Self, Self::Error> {
        let value = value.0;
        Self::try_from(value)
    }
}

impl Display for &GlobalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s: String = (*self).clone().into();
        write!(f, "{}", s)
    }
}
