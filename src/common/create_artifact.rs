use crate::artifact::ArtifactData;
use crate::artifact::ArtifactId;
use crate::artifacts::SeriesPoint;
use crate::project::ProjectId;
use crate::run::RunId;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateArtifactRequest {
    pub project_id: ProjectId,
    pub run_id: Option<RunId>,
    pub artifact_id: ArtifactId,
    pub series_point: Option<SeriesPoint>,
    pub payload: ArtifactData,
}

impl CreateArtifactRequest {
    pub const HTTP_PATH: &'static str = "/create-action";
    pub const HTTP_FIELD_NAME: &'static str = "request";

    pub fn validate(self) -> Result<CreateArtifactRequest, anyhow::Error> {
        if let Some(series_point) = self.series_point.as_ref() {
            if series_point.values.is_empty() {
                return Err(anyhow!("SeriesPoint::values cannot be empty when set"))?;
            }
        }
        Ok(self)
    }
}
