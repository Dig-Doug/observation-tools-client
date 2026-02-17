use crate::group_id::GroupId;
use crate::ExecutionId;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for an observation (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9b")]
pub struct ObservationId(Uuid);

impl Serialize for ObservationId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl ObservationId {
  /// Generate a new UUIDv7 observation ID
  pub fn new() -> Self {
    Self(Uuid::now_v7())
  }

  /// Create a nil (all zeros) observation ID for placeholder use
  pub fn nil() -> Self {
    Self(Uuid::nil())
  }

  /// Parse from a string
  pub fn parse(s: &str) -> crate::Result<Self> {
    let uuid = Uuid::parse_str(s).map_err(crate::Error::InvalidObservationId)?;
    Ok(Self(uuid))
  }
}

impl Default for ObservationId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for ObservationId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.as_simple().to_string())
  }
}

/// An observation is a single piece of collected data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Observation {
  /// Unique identifier for this observation
  pub id: ObservationId,

  /// ID of the execution this observation belongs to
  pub execution_id: ExecutionId,

  /// User-defined name for this observation
  pub name: String,

  /// Type of observation
  pub observation_type: ObservationType,

  /// Log level for this observation
  pub log_level: LogLevel,

  /// Source location where this observation was created
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<SourceInfo>,

  /// User-defined metadata as key-value pairs
  #[serde(default)]
  pub metadata: HashMap<String, String>,

  /// IDs of groups this observation belongs to
  #[serde(default)]
  pub group_ids: Vec<GroupId>,

  /// Parent group ID (used when observation_type == Group)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent_group_id: Option<GroupId>,

  /// Parent span ID (for tracing integration)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent_span_id: Option<String>,

  /// When this observation was created
  pub created_at: DateTime<Utc>,
}

/// Type of observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ObservationType {
  LogEntry,
  Payload,
  Span,
  Group,
}

/// Log level for observations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warning,
  Error,
}

impl From<log::Level> for LogLevel {
  fn from(level: log::Level) -> Self {
    match level {
      log::Level::Trace => LogLevel::Trace,
      log::Level::Debug => LogLevel::Debug,
      log::Level::Info => LogLevel::Info,
      log::Level::Warn => LogLevel::Warning,
      log::Level::Error => LogLevel::Error,
    }
  }
}

/// Source location information for an observation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SourceInfo {
  /// Source file path
  pub file: String,

  /// Line number in the source file
  pub line: u32,

  /// Optional column number
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<u32>,
}
