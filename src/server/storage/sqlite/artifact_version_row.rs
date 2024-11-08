use crate::storage::sqlite::ANCESTORS_SEPARATOR;
use crate::storage::sqlite::ID_PART_SEPARATOR;
use crate::storage::util::ieee_754_total_ordering_value;
use crate::storage::ArtifactVersionRow;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Selectable;
use itertools::Itertools;
use observation_tools_common::artifact::ArtifactData;
use observation_tools_common::artifact::ArtifactId;
use observation_tools_common::artifacts::SeriesDimensionValue;
use observation_tools_common::artifacts::SeriesPoint;
use observation_tools_common::project::ProjectId;
use observation_tools_common::run::RunId;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::storage::sqlite::schema::artifacts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ArtifactVersionSqliteRow {
    pub project_id: Vec<u8>,
    pub run_id: Option<Vec<u8>>,
    pub artifact_id: Vec<u8>,
    pub version_id: Vec<u8>,
    pub artifact_type: String,
    pub version_data: Vec<u8>,
    pub client_creation_time: String,
    pub path: String,
    pub series_id: Option<Vec<u8>>,
    pub series_value: Option<String>,
    pub series_point: Option<Vec<u8>>,
}

impl TryFrom<ArtifactVersionRow> for ArtifactVersionSqliteRow {
    type Error = anyhow::Error;

    fn try_from(value: ArtifactVersionRow) -> Result<Self, Self::Error> {
        let ancestors_list = value
            .version_data
            .ancestor_group_ids
            .iter()
            .map(|id| id.uuid.simple().to_string())
            .join(ID_PART_SEPARATOR);
        Ok(ArtifactVersionSqliteRow {
            project_id: value.project_id.id.as_bytes().to_vec(),
            run_id: value
                .run_id
                .map(|run_id| run_id.id.uuid.as_bytes().to_vec()),
            artifact_id: value.artifact_id.uuid.as_bytes().to_vec(),
            version_id: value.version_id.as_bytes().to_vec(),
            artifact_type: value.version_data.artifact_type.as_string(),
            version_data: rmp_serde::to_vec(&value.version_data)?,
            client_creation_time: value.version_data.client_creation_time.to_rfc3339(),
            path: [ancestors_list, value.artifact_id.uuid.simple().to_string()]
                .join(ANCESTORS_SEPARATOR),
            series_id: value
                .series_point
                .as_ref()
                .map(|series_point| series_point.series_id.artifact_id.uuid.as_bytes().to_vec()),
            series_value: value.series_point.clone().map(|p| {
                let storage: SeriesValueStorage = p.into();
                storage.into()
            }),
            series_point: value
                .series_point
                .map(|series_point| rmp_serde::to_vec(&series_point))
                .transpose()?,
        })
    }
}

impl TryFrom<ArtifactVersionSqliteRow> for ArtifactVersionRow {
    type Error = anyhow::Error;

    fn try_from(value: ArtifactVersionSqliteRow) -> Result<Self, Self::Error> {
        let version_data: ArtifactData = rmp_serde::from_slice(&value.version_data)?;
        let series_point = value
            .series_point
            .map(|series_point| rmp_serde::from_slice(&series_point))
            .transpose()?;
        Ok(ArtifactVersionRow {
            project_id: ProjectId {
                id: Uuid::from_slice(&value.project_id)?,
            },
            run_id: value
                .run_id
                .map(|run_id| {
                    Ok::<_, anyhow::Error>(RunId {
                        id: ArtifactId {
                            uuid: Uuid::from_slice(&run_id)?,
                        },
                    })
                })
                .transpose()?,
            artifact_id: ArtifactId {
                uuid: Uuid::from_slice(&value.artifact_id)?,
            },
            version_id: Uuid::from_slice(&value.version_id)?,
            version_data,
            series_point,
        })
    }
}

pub struct SeriesValueStorage {
    values: Vec<SeriesDimensionValue>,
}

impl From<SeriesPoint> for SeriesValueStorage {
    fn from(value: SeriesPoint) -> Self {
        SeriesValueStorage {
            values: value.values,
        }
    }
}

impl From<SeriesValueStorage> for String {
    fn from(value: SeriesValueStorage) -> Self {
        const SERIES_VALUE_SEPARATOR: &str = ",";
        const SERIES_KEY_VALUE_SEPARATOR: &str = ":";

        value
            .values
            .into_iter()
            .map(|value| {
                format!(
                    "{}{}{}",
                    value.dimension_id.artifact_id.uuid.simple().to_string(),
                    SERIES_KEY_VALUE_SEPARATOR,
                    ieee_754_total_ordering_value(value.value.into())
                )
            })
            .join(SERIES_VALUE_SEPARATOR)
    }
}
