//! Metadata storage for executions and observations

use super::StorageError;
use super::StorageResult;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use std::path::Path;
use tracing::trace;

/// Trait for storing and retrieving execution and observation metadata
#[async_trait::async_trait]
pub trait MetadataStorage: Send + Sync {
  /// Store an execution
  async fn store_execution(&self, execution: &Execution) -> StorageResult<()>;

  /// Get an execution by ID
  async fn get_execution(&self, id: ExecutionId) -> StorageResult<Execution>;

  /// List all executions (with optional pagination)
  async fn list_executions(
    &self,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Execution>>;

  /// Count total number of executions
  async fn count_executions(&self) -> StorageResult<usize>;

  /// Store multiple observations in a batch
  async fn store_observations(&self, observations: &[Observation]) -> StorageResult<()>;

  /// Get observations by their IDs
  async fn get_observations(&self, ids: &[ObservationId]) -> StorageResult<Vec<Observation>>;

  /// List observations for an execution (with optional pagination)
  async fn list_observations(
    &self,
    execution_id: ExecutionId,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Observation>>;

  /// Count total number of observations for an execution
  async fn count_observations(&self, execution_id: ExecutionId) -> StorageResult<usize>;
}

/// Sled-based metadata storage implementation
pub struct SledStorage {
  db: sled::Db,
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
  fn observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("observations")?)
  }

  /// Get the execution->observations index tree
  fn execution_observations_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("execution_observations")?)
  }
}

#[async_trait::async_trait]
impl MetadataStorage for SledStorage {
  async fn store_execution(&self, execution: &Execution) -> StorageResult<()> {
    let tree = self.executions_tree()?;
    let key = execution.id.to_string();
    let value = serde_json::to_vec(execution)?;
    tree.insert(key.as_bytes(), value)?;
    Ok(())
  }

  async fn get_execution(&self, id: ExecutionId) -> StorageResult<Execution> {
    let tree = self.executions_tree()?;
    let key = id.to_string();
    trace!("Retrieving execution: {}", key);
    let value = tree
      .get(key.as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Execution {} not found", id)))?;

    let execution = serde_json::from_slice(&value)?;
    Ok(execution)
  }

  async fn list_executions(
    &self,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Execution>> {
    let tree = self.executions_tree()?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(100);
    let mut executions: Vec<Execution> = tree
      .iter()
      .values()
      .filter_map(|result| result.ok().and_then(|v| serde_json::from_slice(&v).ok()))
      .collect();
    // Sort by created_at descending (most recent first)
    executions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    // Apply pagination after sorting
    let executions: Vec<Execution> = executions.into_iter().skip(offset).take(limit).collect();
    Ok(executions)
  }

  async fn count_executions(&self) -> StorageResult<usize> {
    let tree = self.executions_tree()?;
    Ok(tree.len())
  }

  async fn store_observations(&self, observations: &[Observation]) -> StorageResult<()> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    for observation in observations {
      let key = observation.id.to_string();
      let value = serde_json::to_vec(observation)?;
      obs_tree.insert(key.as_bytes(), value)?;
      // Update the execution->observations index
      let exec_key = format!("{}:{}", observation.execution_id, observation.id);
      trace!("Storing execution-observation index: {}", exec_key);
      exec_obs_tree.insert(exec_key.as_bytes(), observation.id.to_string().as_bytes())?;
    }
    Ok(())
  }

  async fn get_observations(&self, ids: &[ObservationId]) -> StorageResult<Vec<Observation>> {
    let tree = self.observations_tree()?;
    let mut observations = Vec::with_capacity(ids.len());
    for id in ids {
      let key = id.to_string();
      if let Some(value) = tree.get(key.as_bytes())? {
        let observation = serde_json::from_slice(&value)?;
        observations.push(observation);
      }
    }
    Ok(observations)
  }

  async fn list_observations(
    &self,
    execution_id: ExecutionId,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Observation>> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = format!("{}:", execution_id);
    let observations: Vec<Observation> = exec_obs_tree
      .scan_prefix(prefix.as_bytes())
      .values()
      .filter_map(|result| {
        result.ok().and_then(|obs_id| {
          obs_tree
            .get(&obs_id)
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice(&v).ok())
        })
      })
      .skip(offset.unwrap_or(0))
      .take(limit.unwrap_or(100))
      .collect();
    Ok(observations)
  }

  async fn count_observations(&self, execution_id: ExecutionId) -> StorageResult<usize> {
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = format!("{}:", execution_id);
    let count = exec_obs_tree
      .scan_prefix(prefix.as_bytes())
      .count();
    Ok(count)
  }
}
