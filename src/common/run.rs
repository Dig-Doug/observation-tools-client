use crate::artifact::ArtifactId;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunId {
    pub id: ArtifactId,
}
