use crate::storage::project::ProjectRow;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Selectable;
use observation_tools_common::project::ProjectId;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::storage::sqlite::schema::projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProjectSqliteRow {
    pub id: Vec<u8>,
    pub data: Vec<u8>,
}

impl TryFrom<ProjectRow> for ProjectSqliteRow {
    type Error = anyhow::Error;

    fn try_from(value: ProjectRow) -> Result<Self, Self::Error> {
        Ok(ProjectSqliteRow {
            id: value.id.uuid.as_bytes().to_vec(),
            data: rmp_serde::to_vec(&value.data)?,
        })
    }
}

impl TryFrom<ProjectSqliteRow> for ProjectRow {
    type Error = anyhow::Error;

    fn try_from(value: ProjectSqliteRow) -> Result<Self, Self::Error> {
        Ok(ProjectRow {
            id: ProjectId {
                uuid: Uuid::from_slice(&value.id)?,
            },
            data: rmp_serde::from_slice(&value.data)?,
        })
    }
}
