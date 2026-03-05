//! Sled-based metadata storage implementation

mod executions;
pub(crate) mod group;
pub(crate) mod keys;
mod observations;
mod payloads;

use super::StorageResult;
use observation_tools_shared::ObservationId;
use std::path::Path;

/// Sled-based metadata storage implementation
pub struct SledStorage {
  db: sled::Db,
}

/// Sentinel value for root-level items in the group_children tree
const ROOT_SENTINEL: &str = "_ROOT_";

pub(crate) fn metadata_key(obs_id: &ObservationId) -> String {
  obs_id.to_string()
}

impl SledStorage {
  /// Create a new Sled storage instance
  pub fn new(path: impl AsRef<Path>) -> StorageResult<Self> {
    let db = sled::open(path)?;
    Ok(Self { db })
  }

  /// Get the executions tree
  pub(super) fn executions_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("executions")?)
  }

  /// Get the observations tree
  pub(crate) fn observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("observations")?)
  }

  /// Get the execution->observations index tree
  pub(super) fn execution_observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("execution_observations")?)
  }

  /// Get the inline payloads tree
  pub(super) fn payloads_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("payloads")?)
  }

  /// Get the group_children index tree
  pub(crate) fn group_children_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("group_children")?)
  }

  /// Get the group_index tree (group_id → observation_id)
  pub(crate) fn group_index_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("group_index")?)
  }
}

#[cfg(test)]
pub(super) mod test_helpers {
  use super::SledStorage;
  use observation_tools_shared::{
    Execution, ExecutionId, GroupId, LogLevel, Observation, ObservationId, ObservationType,
  };
  use tempfile::TempDir;

  pub fn test_storage() -> (SledStorage, TempDir) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let storage = SledStorage::new(dir.path()).expect("failed to create storage");
    (storage, dir)
  }

  pub fn make_execution() -> Execution {
    Execution {
      id: ExecutionId::new(),
      name: "test".to_string(),
      metadata: Default::default(),
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    }
  }

  pub fn make_observation(exec_id: ExecutionId, name: &str) -> Observation {
    Observation {
      id: ObservationId::new(),
      execution_id: exec_id,
      name: name.to_string(),
      observation_type: ObservationType::LogEntry,
      log_level: LogLevel::Info,
      source: None,
      metadata: Default::default(),
      group_ids: Vec::new(),
      parent_group_id: None,
      parent_span_id: None,
      created_at: chrono::Utc::now(),
    }
  }

  pub fn make_group(exec_id: ExecutionId, name: &str, parent: Option<GroupId>) -> Observation {
    let id = ObservationId::new();
    let group_id = GroupId::from(id.to_string());
    Observation {
      id,
      execution_id: exec_id,
      name: name.to_string(),
      observation_type: ObservationType::Group,
      log_level: LogLevel::Info,
      source: None,
      metadata: Default::default(),
      group_ids: vec![group_id],
      parent_group_id: parent,
      parent_span_id: None,
      created_at: chrono::Utc::now(),
    }
  }

  pub fn make_obs_in_group(exec_id: ExecutionId, name: &str, group_id: GroupId) -> Observation {
    Observation {
      id: ObservationId::new(),
      execution_id: exec_id,
      name: name.to_string(),
      observation_type: ObservationType::LogEntry,
      log_level: LogLevel::Info,
      source: None,
      metadata: Default::default(),
      group_ids: vec![group_id],
      parent_group_id: None,
      parent_span_id: None,
      created_at: chrono::Utc::now(),
    }
  }
}
