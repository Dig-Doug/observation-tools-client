use crate::artifact::ArtifactData;
use crate::artifact::ArtifactId;
use crate::artifact::ArtifactUpdate;
use crate::artifacts::SeriesPoint;
use crate::project::ProjectId;
use crate::run::RunId;
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct CreateArtifactRequest {
    project_id: ProjectId,
    run_id: RunId,
    artifact_id: ArtifactId,
    series_point: Option<SeriesPoint>,
    payload: CreateArtifactPayload,
}

#[derive(Debug, Clone)]
pub enum CreateArtifactPayload {
    Create(ArtifactData),
    Update(ArtifactUpdate),
}

impl CreateArtifactRequest {
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

#[derive(Debug, Clone)]
pub struct CreateArtifactResponse {}
