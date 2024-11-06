pub mod schema;

use crate::storage::ArtifactVersion;
use anyhow::anyhow;
use axum::body::Bytes;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use futures_util::TryStream;
use futures_util::TryStreamExt;
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::artifacts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ArtifactVersionRow {
    pub project_id: Vec<u8>,
    pub run_id: Option<Vec<u8>>,
    pub artifact_id: Vec<u8>,
    pub version_id: Vec<u8>,
    pub artifact_type: i32,
    pub proto_data: Vec<u8>,
    pub client_creation_time: String,
    pub path: String,
    pub series_id: Option<Vec<u8>>,
    pub series_value: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::payloads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PayloadRow {
    pub project_id: Vec<u8>,
    pub artifact_id: Vec<u8>,
    pub version_id: Vec<u8>,
    pub payload: Vec<u8>,
}

pub const ID_PART_SEPARATOR: &str = ",";
pub const ANCESTORS_SEPARATOR: &str = "-";
pub const ANCESTORS_SEPARATOR_NEXT: &str = ".";

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

    pub async fn write_artifact_version<E: Error + Send + Sync + 'static>(
        &self,
        version: ArtifactVersion,
        bytes_stream: impl TryStream<Ok = Bytes, Error = E> + Unpin,
    ) -> Result<(), anyhow::Error> {
        let all_bytes = bytes_stream
            .try_fold(Vec::new(), |mut acc, bytes| async move {
                acc.extend_from_slice(&bytes);
                Ok(acc)
            })
            .await?;
        todo!("Impl")
    }
}
