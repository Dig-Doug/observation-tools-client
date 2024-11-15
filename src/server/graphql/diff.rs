use crate::graphql::artifact_version::ArtifactVersionDataLoader;
use crate::storage::artifact::ArtifactStorage;
use async_graphql::Context;
use async_graphql::Object;
use async_graphql::ID;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use similar::TextDiff;

#[derive(Default)]
pub struct DiffArtifactsQuery {}

#[Object]
impl DiffArtifactsQuery {
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

        let storage = ctx.data::<ArtifactStorage>()?;
        let left_payload = storage
            .read_artifact_version_payload(&left_version.row.absolute_id())
            .await?;
        let left_text = left_payload
            .map(|p| serde_json::to_string_pretty(&p))
            .transpose()?;
        let right_payload = storage
            .read_artifact_version_payload(&right_version.row.absolute_id())
            .await?;
        let right_text = right_payload
            .map(|p| serde_json::to_string_pretty(&p))
            .transpose()?;

        let text_diff = TextDiff::from_lines(
            &left_text.unwrap_or_default(),
            &right_text.unwrap_or_default(),
        )
        .unified_diff()
        .header(&left.0, &right.0)
        .to_string();
        Ok(text_diff)
    }
}
