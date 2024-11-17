use crate::storage::sqlite::artifact_version_row::ArtifactVersionSqliteRow;
use crate::storage::sqlite::schema;
use crate::storage::sqlite::PayloadRow;
use crate::storage::sqlite::SqliteStorage;
use crate::storage::ArtifactVersionRow;
use crate::storage::ArtifactVersionRowOrError;
use axum::body::Bytes;
use diesel::dsl::insert_into;
use diesel::prelude::*;
use futures_util::TryStream;
use futures_util::TryStreamExt;
use itertools::Itertools;
use observation_tools_common::artifact::AbsoluteArtifactVersionId;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifact::ArtifactVersionId;
use observation_tools_common::artifact::StructuredData;
use observation_tools_common::project::ProjectId;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

impl SqliteStorage {
    pub async fn read_artifact_versions(
        &self,
        versions: &Vec<AbsoluteArtifactVersionId>,
    ) -> Result<HashMap<AbsoluteArtifactVersionId, ArtifactVersionRowOrError>, anyhow::Error> {
        let mut connection = self.pool.get()?;
        let version_rows = schema::artifacts::table
            // TODO(doug): Technically, this query could return collisions because we are not
            // building a filter that checks for all three field values at once. However, since
            // we're using UUIDs, this should be practically impossible.
            .filter(
                schema::artifacts::project_id
                    .eq_any(
                        versions
                            .iter()
                            .map(|id| id.project_id.uuid.as_bytes().to_vec())
                            .collect_vec(),
                    )
                    .and(
                        schema::artifacts::artifact_id.eq_any(
                            versions
                                .iter()
                                .map(|id| id.artifact_id.uuid.as_bytes().to_vec())
                                .collect_vec(),
                        ),
                    )
                    .and(
                        schema::artifacts::version_id.eq_any(
                            versions
                                .iter()
                                .map(|id| id.version_id.uuid.as_bytes().to_vec())
                                .collect_vec(),
                        ),
                    ),
            )
            .load::<ArtifactVersionSqliteRow>(&mut connection)?;
        let version_rows: Result<
            HashMap<AbsoluteArtifactVersionId, ArtifactVersionRowOrError>,
            anyhow::Error,
        > = version_rows
            .into_iter()
            .map(|row| {
                let version_id = AbsoluteArtifactVersionId {
                    project_id: ProjectId {
                        uuid: Uuid::from_slice(&row.project_id)?,
                    },
                    artifact_id: ArtifactId {
                        uuid: Uuid::from_slice(&row.artifact_id)?,
                    },
                    version_id: ArtifactVersionId {
                        uuid: Uuid::from_slice(&row.version_id)?,
                    },
                };
                let version_row = ArtifactVersionRow::try_from(row);
                Ok((version_id, version_row))
            })
            .collect();
        Ok(version_rows?)
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
                    project_id: version.project_id.uuid.as_bytes().to_vec(),
                    artifact_id: version.artifact_id.uuid.as_bytes().to_vec(),
                    version_id: version.version_id.uuid.as_bytes().to_vec(),
                    payload: all_bytes,
                })
                .execute(&mut connection)?;
        }
        insert_into(schema::artifacts::table)
            .values(ArtifactVersionSqliteRow::try_from(version)?)
            .execute(&mut connection)?;
        Ok(())
    }

    pub async fn read_artifact_version_payload(
        &self,
        version_id: &AbsoluteArtifactVersionId,
    ) -> Result<Option<StructuredData>, anyhow::Error> {
        let mut connection = self.pool.get()?;
        let payload = schema::payloads::table
            .filter(
                schema::payloads::project_id
                    .eq(version_id.project_id.uuid.as_bytes().to_vec())
                    .and(
                        schema::payloads::artifact_id.eq(version_id
                            .artifact_id
                            .uuid
                            .as_bytes()
                            .to_vec()),
                    )
                    .and(
                        schema::payloads::version_id.eq(version_id
                            .version_id
                            .uuid
                            .as_bytes()
                            .to_vec()),
                    ),
            )
            .select(schema::payloads::payload)
            .first::<Vec<u8>>(&mut connection)
            .optional()?;
        match payload {
            Some(payload) => Ok(Some(rmp_serde::from_slice(&payload)?)),
            None => Ok(None),
        }
    }
}
