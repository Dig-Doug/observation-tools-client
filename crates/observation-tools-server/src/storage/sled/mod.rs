//! Sled-based metadata storage implementation

mod executions;
mod groups;
pub(crate) mod keys;
mod observations;
mod payloads;

use super::GroupDirectDescendantsPage;
use super::GroupTree;
use super::MetadataStorage;
use super::ObservationPage;
use super::ObservationPayloadPage;
use super::ObservationWithPayloads;
use super::StorageResult;
use super::StoredPayload;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
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
  fn executions_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("executions")?)
  }

  /// Get the observations tree
  pub(crate) fn observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("observations")?)
  }

  /// Get the execution->observations index tree
  fn execution_observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("execution_observations")?)
  }

  /// Get the inline payloads tree
  fn payloads_tree(&self) -> StorageResult<sled::Tree> {
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

#[async_trait::async_trait]
impl MetadataStorage for SledStorage {
  async fn store_execution(&self, execution: &Execution) -> StorageResult<()> {
    self.store_execution_impl(execution)
  }

  async fn get_execution(&self, id: ExecutionId) -> StorageResult<Execution> {
    self.get_execution_impl(id)
  }

  async fn list_executions(
    &self,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Execution>> {
    self.list_executions_impl(limit, offset)
  }

  async fn count_executions(&self) -> StorageResult<usize> {
    self.count_executions_impl()
  }

  async fn store_observations(
    &self,
    observations: Vec<observation_tools_shared::Observation>,
  ) -> StorageResult<()> {
    self.store_observations_impl(observations)
  }

  async fn store_payloads(
    &self,
    observation_id: &ObservationId,
    payloads: &[StoredPayload],
  ) -> StorageResult<()> {
    self.store_payloads_impl(observation_id, payloads)
  }

  async fn get_observation(
    &self,
    id: ObservationId,
  ) -> StorageResult<observation_tools_shared::Observation> {
    self.get_observation_impl(id)
  }

  async fn get_all_payloads(
    &self,
    observation_id: ObservationId,
  ) -> StorageResult<Vec<StoredPayload>> {
    self.get_all_payloads_impl(observation_id)
  }

  async fn get_observations(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<ObservationPage> {
    self.get_observations_impl(execution_id, page_token, observation_type)
  }

  async fn get_payloads(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage> {
    self.get_payloads_impl(execution_id, observation_id, page_token)
  }

  async fn get_direct_descendants_page(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    page_token: Option<String>,
  ) -> StorageResult<GroupDirectDescendantsPage> {
    self.get_direct_descendants_page_impl(execution_id, group_id, page_token)
  }

  async fn get_observation_by_group_id(
    &self,
    group_id: GroupId,
  ) -> StorageResult<ObservationWithPayloads> {
    self.get_observation_by_group_id_impl(group_id)
  }

  async fn get_group_tree_bfs(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    max_nodes: usize,
  ) -> StorageResult<GroupTree> {
    self.get_group_tree_bfs_impl(execution_id, group_id, max_nodes)
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
    Observation {
      id: ObservationId::new(),
      execution_id: exec_id,
      name: name.to_string(),
      observation_type: ObservationType::Group,
      log_level: LogLevel::Info,
      source: None,
      metadata: Default::default(),
      group_ids: Vec::new(),
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
