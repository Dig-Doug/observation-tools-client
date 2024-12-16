use crate::graphql::artifact_version::ArtifactVersionDataLoader;
use crate::storage::artifact::Storage;
use async_graphql::Context;
use async_graphql::Object;
use async_graphql::ID;
use observation_tools_common::artifact::{AbsoluteArtifactVersionId, StructuredData};
use observation_tools_common::artifacts::Object1Data;
use similar::TextDiff;
use tracing::{info, trace};

#[derive(Default)]
pub struct DiffArtifactsQuery {}

#[Object]
impl DiffArtifactsQuery {
    #[tracing::instrument(skip_all)]
    async fn diff_artifacts(
        &self,
        ctx: &Context<'_>,
        left: ID,
        right: ID,
    ) -> async_graphql::Result<String> {
        if left == right {
            return Ok("No diff".to_string());
        }
        let left_id: AbsoluteArtifactVersionId = left.clone().try_into()?;
        let right_id: AbsoluteArtifactVersionId = right.clone().try_into()?;

        let loader = ctx.data::<ArtifactVersionDataLoader>()?;
        let mut doc = loader
            .load_many([left_id.clone(), right_id.clone()])
            .await?;
        let left_version = doc.remove(&left_id).ok_or("Left not found")??;
        let right_version = doc.remove(&right_id).ok_or("Right not found")??;

        let storage = ctx.data::<Storage>()?;
        let left_payload = storage
            .read_artifact_version_payload(&left_version.row.absolute_id())
            .await?;
        let right_payload = storage
            .read_artifact_version_payload(&right_version.row.absolute_id())
            .await?;
        Ok(diff_structured_data(left_payload, right_payload)?)
    }
}

pub fn get_text_content(structured_data: StructuredData) -> Result<String, anyhow::Error> {
    match &structured_data {
        StructuredData::Object1(obj) => match &obj.data {
            Object1Data::Text(t) => return Ok(t.text.clone()),
            _ => {}
        },
        _ => {}
    };
    Ok(serde_json::to_string_pretty(&structured_data)?)
}

#[tracing::instrument(skip_all)]
fn diff_structured_data(
    left: Option<StructuredData>,
    right: Option<StructuredData>,
) -> Result<String, anyhow::Error> {
    trace!("Generating diff...");
    let left_text = left.map(|p| get_text_content(p)).transpose()?;
    let right_text = right.map(|p| get_text_content(p)).transpose()?;
    let text_diff = TextDiff::from_lines(
        &left_text.unwrap_or_default(),
        &right_text.unwrap_or_default(),
    )
    .unified_diff()
    .header("left", "right")
    .to_string();
    Ok(text_diff)
}
