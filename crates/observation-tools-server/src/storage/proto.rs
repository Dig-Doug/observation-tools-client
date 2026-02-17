use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::LogLevel;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::PayloadId;
use observation_tools_shared::SourceInfo;
use std::collections::HashMap;

use super::StorageError;

/// Protobuf-encoded observation metadata + payload manifest
#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredObservation {
  /// ObservationId as simple UUID string
  #[prost(string, tag = "1")]
  pub id: String,
  /// ExecutionId as simple UUID string
  #[prost(string, tag = "2")]
  pub execution_id: String,
  #[prost(string, tag = "3")]
  pub name: String,
  /// ObservationType as i32
  #[prost(int32, tag = "4")]
  pub observation_type: i32,
  /// LogLevel as i32
  #[prost(int32, tag = "5")]
  pub log_level: i32,
  #[prost(message, optional, tag = "6")]
  pub source: Option<StoredSourceInfo>,
  /// Metadata as repeated key-value pairs
  #[prost(message, repeated, tag = "7")]
  pub metadata: Vec<StoredKeyValue>,
  /// Group IDs as strings
  #[prost(string, repeated, tag = "8")]
  pub group_ids: Vec<String>,
  /// Parent group ID (optional)
  #[prost(string, optional, tag = "9")]
  pub parent_group_id: Option<String>,
  /// Parent span ID (optional)
  #[prost(string, optional, tag = "10")]
  pub parent_span_id: Option<String>,
  /// Created at as RFC3339 string
  #[prost(string, tag = "11")]
  pub created_at: String,
  /// MIME type of the primary payload (kept for backward compat in Observation)
  #[prost(string, tag = "12")]
  pub mime_type: String,
  /// Size of the primary payload in bytes
  #[prost(uint64, tag = "13")]
  pub payload_size: u64,
  /// Payload manifest: metadata about all payloads for this observation
  #[prost(message, repeated, tag = "14")]
  pub payload_manifest: Vec<StoredPayloadMeta>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredSourceInfo {
  #[prost(string, tag = "1")]
  pub file: String,
  #[prost(uint32, tag = "2")]
  pub line: u32,
  #[prost(uint32, optional, tag = "3")]
  pub column: Option<u32>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredKeyValue {
  #[prost(string, tag = "1")]
  pub key: String,
  #[prost(string, tag = "2")]
  pub value: String,
}

/// Metadata about a single payload attached to an observation
#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredPayloadMeta {
  /// PayloadId as simple UUID string
  #[prost(string, tag = "1")]
  pub payload_id: String,
  /// Name of the payload (e.g., "default", "headers", "body")
  #[prost(string, tag = "2")]
  pub name: String,
  /// MIME type
  #[prost(string, tag = "3")]
  pub mime_type: String,
  /// Size in bytes
  #[prost(uint64, tag = "4")]
  pub size: u64,
  /// Whether this payload is stored as a blob (true) or inline (false)
  #[prost(bool, tag = "5")]
  pub is_blob: bool,
}

/// Protobuf-encoded inline payload data
#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredInlinePayload {
  #[prost(bytes = "vec", tag = "1")]
  pub data: Vec<u8>,
}

// Conversion: Observation -> StoredObservation (without payloads, those are set separately)
impl StoredObservation {
  pub fn from_observation(obs: &Observation) -> Self {
    StoredObservation {
      id: obs.id.to_string(),
      execution_id: obs.execution_id.to_string(),
      name: obs.name.clone(),
      observation_type: observation_type_to_i32(obs.observation_type),
      log_level: log_level_to_i32(obs.log_level),
      source: obs.source.as_ref().map(|s| StoredSourceInfo {
        file: s.file.clone(),
        line: s.line,
        column: s.column,
      }),
      metadata: obs
        .metadata
        .iter()
        .map(|(k, v)| StoredKeyValue {
          key: k.clone(),
          value: v.clone(),
        })
        .collect(),
      group_ids: obs.group_ids.iter().map(|g| g.to_string()).collect(),
      parent_group_id: obs.parent_group_id.as_ref().map(|g| g.to_string()),
      parent_span_id: obs.parent_span_id.clone(),
      created_at: obs.created_at.to_rfc3339(),
      mime_type: String::new(),
      payload_size: 0,
      payload_manifest: Vec::new(),
    }
  }

  pub fn to_observation(&self) -> Result<Observation, StorageError> {
    let id = ObservationId::parse(&self.id)
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;
    let execution_id = ExecutionId::parse(&self.execution_id)
      .map_err(|e| StorageError::Internal(format!("Invalid execution ID: {}", e)))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
      .map_err(|e| StorageError::Internal(format!("Invalid created_at: {}", e)))?
      .with_timezone(&chrono::Utc);

    let mut metadata = HashMap::new();
    for kv in &self.metadata {
      metadata.insert(kv.key.clone(), kv.value.clone());
    }

    Ok(Observation {
      id,
      execution_id,
      name: self.name.clone(),
      observation_type: observation_type_from_i32(self.observation_type),
      log_level: log_level_from_i32(self.log_level),
      source: self.source.as_ref().map(|s| SourceInfo {
        file: s.file.clone(),
        line: s.line,
        column: s.column,
      }),
      metadata,
      group_ids: self.group_ids.iter().map(|g| GroupId::parse(g)).collect(),
      parent_group_id: self.parent_group_id.as_ref().map(|g| GroupId::parse(g)),
      parent_span_id: self.parent_span_id.clone(),
      created_at,
    })
  }
}

impl StoredPayloadMeta {
  pub fn payload_id(&self) -> Result<PayloadId, StorageError> {
    PayloadId::parse(&self.payload_id)
      .map_err(|e| StorageError::Internal(format!("Invalid payload ID: {}", e)))
  }
}

fn observation_type_to_i32(t: ObservationType) -> i32 {
  match t {
    ObservationType::LogEntry => 0,
    ObservationType::Payload => 1,
    ObservationType::Span => 2,
    ObservationType::Group => 3,
  }
}

fn observation_type_from_i32(v: i32) -> ObservationType {
  match v {
    0 => ObservationType::LogEntry,
    1 => ObservationType::Payload,
    2 => ObservationType::Span,
    3 => ObservationType::Group,
    _ => ObservationType::Payload,
  }
}

fn log_level_to_i32(l: LogLevel) -> i32 {
  match l {
    LogLevel::Trace => 0,
    LogLevel::Debug => 1,
    LogLevel::Info => 2,
    LogLevel::Warning => 3,
    LogLevel::Error => 4,
  }
}

fn log_level_from_i32(v: i32) -> LogLevel {
  match v {
    0 => LogLevel::Trace,
    1 => LogLevel::Debug,
    2 => LogLevel::Info,
    3 => LogLevel::Warning,
    4 => LogLevel::Error,
    _ => LogLevel::Info,
  }
}
