use crate::graphql::artifact::ArtifactVersion;
use crate::graphql::artifact::ArtifactVersionDataLoader;
use crate::graphql::project::Project;
use crate::graphql::project::ProjectDataLoader;
use anyhow::anyhow;
use async_graphql::dataloader::DataLoader;
use async_graphql::Context;
use async_graphql::Interface;
use async_graphql::Object;
use async_graphql::ID;
use observation_tools_common::GlobalId;

#[derive(Interface)]
#[graphql(field(name = "id", ty = "ID"))]
enum Node {
    Project(Project),
    ArtifactVersion(ArtifactVersion),
}

#[derive(Default)]
pub struct GetNodeQuery {}

#[Object]
impl GetNodeQuery {
    async fn node(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Node> {
        let global_id: GlobalId = id.try_into()?;
        match global_id {
            GlobalId::Project(project_id) => {
                let loader = ctx.data::<ProjectDataLoader>()?;
                let doc = loader
                    .load_one(project_id)
                    .await?
                    .ok_or(anyhow!("Person not found"))?;
                Ok(Node::Project(doc?))
            }
            GlobalId::ArtifactVersion(version_id) => {
                let loader = ctx.data::<ArtifactVersionDataLoader>()?;
                let doc = loader
                    .load_one(version_id)
                    .await?
                    .ok_or(anyhow!("Document not found"))?;
                Ok(Node::ArtifactVersion(doc?))
            }
            GlobalId::Artifact(_) => Err("Implement Artifact node".to_string())?,
        }
    }
}
