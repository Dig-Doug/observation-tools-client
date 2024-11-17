mod artifact_version_row;
mod artifacts;
pub mod permission_row;
mod permissions;
mod project_row;
mod projects;
pub mod schema;

use anyhow::anyhow;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use futures_util::TryStreamExt;

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
pub struct SqliteStorage {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteStorage {
    pub fn init(&self) -> Result<(), anyhow::Error> {
        let mut connection = self.pool.get()?;
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("Failed to apply migrations: {}", e))?;
        Ok(())
    }
}
