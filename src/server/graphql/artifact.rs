use crate::storage::ArtifactVersionRow;
use async_graphql::Object;
use async_graphql::ID;

#[derive(Clone, Debug)]
pub struct ArtifactVersion {
    pub row: ArtifactVersionRow,
}

#[Object]
impl ArtifactVersion {
    pub async fn id(&self) -> async_graphql::Result<ID> {
        Ok(self.row.global_id().into())
    }

    pub async fn json(&self) -> async_graphql::Result<String> {
        Ok(serde_json::to_string(&self.row)?)
    }
}
