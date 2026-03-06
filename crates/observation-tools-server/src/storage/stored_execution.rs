//! Protobuf-encoded execution storage type

use super::proto::StoredKeyValue;
use super::StorageError;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use std::collections::HashMap;

/// Protobuf-encoded execution metadata
#[derive(Clone, PartialEq, prost::Message)]
pub struct StoredExecution {
  /// ExecutionId as UUID string
  #[prost(string, tag = "1")]
  pub id: String,
  #[prost(string, tag = "2")]
  pub name: String,
  /// Metadata as repeated key-value pairs
  #[prost(message, repeated, tag = "3")]
  pub metadata: Vec<StoredKeyValue>,
  /// Created at as RFC3339 string
  #[prost(string, tag = "4")]
  pub created_at: String,
  /// Updated at as RFC3339 string
  #[prost(string, tag = "5")]
  pub updated_at: String,
}

impl StoredExecution {
  pub fn from_execution(exec: &Execution) -> Self {
    StoredExecution {
      id: exec.id.to_string(),
      name: exec.name.clone(),
      metadata: exec
        .metadata
        .iter()
        .map(|(k, v)| StoredKeyValue {
          key: k.clone(),
          value: v.clone(),
        })
        .collect(),
      created_at: exec.created_at.to_rfc3339(),
      updated_at: exec.updated_at.to_rfc3339(),
    }
  }

  pub fn to_execution(&self) -> Result<Execution, StorageError> {
    let id = ExecutionId::parse(&self.id)
      .map_err(|e| StorageError::Internal(format!("Invalid execution ID: {}", e)))?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
      .map_err(|e| StorageError::Internal(format!("Invalid created_at: {}", e)))?
      .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
      .map_err(|e| StorageError::Internal(format!("Invalid updated_at: {}", e)))?
      .with_timezone(&chrono::Utc);

    let mut metadata = HashMap::new();
    for kv in &self.metadata {
      metadata.insert(kv.key.clone(), kv.value.clone());
    }

    Ok(Execution {
      id,
      name: self.name.clone(),
      metadata,
      created_at,
      updated_at,
    })
  }
}
