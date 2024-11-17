use crate::auth::permission::Permission;
use crate::auth::resource_id::ResourceType;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Selectable;
use observation_tools_common::artifact::AbsoluteArtifactId;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::project::ProjectId;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::storage::sqlite::schema::permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PermissionSqliteRow {
    pub principal_id: String,
    pub resource_type: i32,
    pub relation: i32,
    pub project_id: Vec<u8>,
    pub artifact_id: Option<Vec<u8>>,
}

impl From<Permission<ProjectId>> for PermissionSqliteRow {
    fn from(value: Permission<ProjectId>) -> Self {
        PermissionSqliteRow {
            principal_id: value.principal.0,
            resource_type: ResourceType::Project as i32,
            relation: value.operation as i32,
            project_id: value.resource_id.uuid.as_bytes().to_vec(),
            artifact_id: None,
        }
    }
}

impl From<Permission<AbsoluteArtifactId>> for PermissionSqliteRow {
    fn from(value: Permission<AbsoluteArtifactId>) -> Self {
        PermissionSqliteRow {
            principal_id: value.principal.0,
            resource_type: ResourceType::Artifact as i32,
            relation: value.operation as i32,
            project_id: value.resource_id.project_id.uuid.as_bytes().to_vec(),
            artifact_id: Some(
                value
                    .resource_id
                    .artifact_id
                    .uuid
                    .as_bytes()
                    .to_vec()
                    .into(),
            ),
        }
    }
}

impl TryFrom<PermissionSqliteRow> for Permission<ProjectId> {
    type Error = anyhow::Error;

    fn try_from(value: PermissionSqliteRow) -> Result<Self, Self::Error> {
        Ok(Permission {
            principal: value.principal_id.into(),
            resource_id: ProjectId {
                uuid: Uuid::from_slice(&value.project_id)?,
            },
            operation: value.relation.try_into()?,
        })
    }
}

impl TryFrom<PermissionSqliteRow> for Permission<AbsoluteArtifactId> {
    type Error = anyhow::Error;

    fn try_from(value: PermissionSqliteRow) -> Result<Self, Self::Error> {
        Ok(Permission {
            principal: value.principal_id.into(),
            resource_id: AbsoluteArtifactId {
                project_id: ProjectId {
                    uuid: Uuid::from_slice(&value.project_id)?,
                },
                artifact_id: ArtifactId {
                    uuid: Uuid::from_slice(
                        &value
                            .artifact_id
                            .ok_or_else(|| anyhow::anyhow!("Artifact ID is missing"))?,
                    )?,
                },
            },
            operation: value.relation.try_into()?,
        })
    }
}
