mod artifact_version_row;
mod project_row;
pub mod schema;

use crate::storage::project::ProjectRow;
use crate::storage::project::ProjectRowOrError;
use crate::storage::sqlite::artifact_version_row::ArtifactVersionSqliteRow;
use crate::storage::sqlite::project_row::ProjectSqliteRow;
use crate::storage::ArtifactVersionRow;
use anyhow::anyhow;
use axum::body::Bytes;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use futures_util::TryStream;
use futures_util::TryStreamExt;
use itertools::Itertools;
use observation_tools_common::project::ProjectId;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("storage/sqlite/migrations");
const ID_PART_SEPARATOR: &str = ",";
const ANCESTORS_SEPARATOR: &str = "-";

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::payloads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PayloadRow {
    pub project_id: Vec<u8>,
    pub artifact_id: Vec<u8>,
    pub version_id: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Clone)]
pub struct SqliteArtifactStorage {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteArtifactStorage {
    pub fn init(&self) -> Result<(), anyhow::Error> {
        let mut connection = self.pool.get()?;
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("Failed to apply migrations: {}", e))?;
        Ok(())
    }

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
        projects: Vec<ProjectId>,
    ) -> Result<HashMap<ProjectId, ProjectRowOrError>, anyhow::Error> {
        let mut connection = self.pool.get()?;
        let project_rows = schema::projects::table
            .filter(
                schema::projects::id.eq_any(
                    projects
                        .iter()
                        .map(|id| id.id.as_bytes().to_vec())
                        .collect_vec(),
                ),
            )
            .load::<ProjectSqliteRow>(&mut connection)?;
        let project_rows: Result<HashMap<ProjectId, ProjectRowOrError>, anyhow::Error> =
            project_rows
                .into_iter()
                .map(|row| {
                    let project_id = ProjectId {
                        id: Uuid::from_slice(&row.id)?,
                    };
                    let project_row = ProjectRow::try_from(row);
                    Ok((project_id, project_row))
                })
                .collect();
        Ok(project_rows?)
    }

    pub async fn write_artifact_version<E: Error + Send + Sync + 'static>(
        &self,
        version: ArtifactVersionRow,
        bytes_stream: Option<impl TryStream<Ok = Bytes, Error = E> + Unpin>,
    ) -> Result<(), anyhow::Error> {
        let mut connection = self.pool.get()?;
        if let Some(bytes_stream) = bytes_stream {
            let all_bytes = bytes_stream
                .try_fold(Vec::new(), |mut acc, bytes| async move {
                    acc.extend_from_slice(&bytes);
                    Ok(acc)
                })
                .await?;
            insert_into(schema::payloads::table)
                .values(PayloadRow {
                    project_id: version.project_id.id.as_bytes().to_vec(),
                    artifact_id: version.artifact_id.uuid.as_bytes().to_vec(),
                    version_id: version.version_id.as_bytes().to_vec(),
                    payload: all_bytes,
                })
                .execute(&mut connection)?;
        }
        insert_into(schema::artifacts::table)
            .values(ArtifactVersionSqliteRow::try_from(version)?)
            .execute(&mut connection)?;
        Ok(())
    }
}
