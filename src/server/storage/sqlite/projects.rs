use crate::storage::project::ProjectRow;
use crate::storage::project::ProjectRowOrError;
use crate::storage::sqlite::project_row::ProjectSqliteRow;
use crate::storage::sqlite::schema;
use crate::storage::sqlite::SqliteStorage;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use itertools::Itertools;
use observation_tools_common::project::ProjectId;
use std::collections::HashMap;
use uuid::Uuid;

impl SqliteStorage {
    pub async fn create_project(&self, project_row: ProjectRow) -> Result<(), anyhow::Error> {
        let mut connection = self.pool.get()?;
        let project_row = ProjectSqliteRow::try_from(project_row)?;
        insert_into(schema::projects::table)
            .values(project_row)
            .execute(&mut connection)?;
        Ok(())
    }

    pub async fn read_projects(
        &self,
        projects: &Vec<ProjectId>,
    ) -> Result<HashMap<ProjectId, ProjectRowOrError>, anyhow::Error> {
        let mut connection = self.pool.get()?;
        let project_rows = schema::projects::table
            .filter(
                schema::projects::id.eq_any(
                    projects
                        .iter()
                        .map(|id| id.uuid.as_bytes().to_vec())
                        .collect_vec(),
                ),
            )
            .load::<ProjectSqliteRow>(&mut connection)?;
        let project_rows: Result<HashMap<ProjectId, ProjectRowOrError>, anyhow::Error> =
            project_rows
                .into_iter()
                .map(|row| {
                    let project_id = ProjectId {
                        uuid: Uuid::from_slice(&row.id)?,
                    };
                    let project_row = ProjectRow::try_from(row);
                    Ok((project_id, project_row))
                })
                .collect();
        Ok(project_rows?)
    }
}
