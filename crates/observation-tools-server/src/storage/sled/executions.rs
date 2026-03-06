//! Execution storage methods for SledStorage

use super::SledStorage;
use crate::storage::stored_execution::StoredExecution;
use crate::storage::ExecutionStorage;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use prost::Message;
use tracing::trace;

#[async_trait::async_trait]
impl ExecutionStorage for SledStorage {
  async fn store_execution(&self, execution: &Execution) -> StorageResult<()> {
    let tree = self.executions_tree()?;
    let key = execution.id.to_string();
    let stored = StoredExecution::from_execution(execution);
    tree.insert(key.as_bytes(), stored.encode_to_vec())?;
    Ok(())
  }

  async fn get_execution(&self, id: ExecutionId) -> StorageResult<Execution> {
    let tree = self.executions_tree()?;
    let key = id.to_string();
    trace!("Retrieving execution: {}", key);
    let value = tree
      .get(key.as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Execution {} not found", id)))?;

    let stored = StoredExecution::decode(value.as_ref())?;
    stored.to_execution()
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
      .filter_map(|result| {
        result
          .ok()
          .and_then(|v| StoredExecution::decode(v.as_ref()).ok())
          .and_then(|s| s.to_execution().ok())
      })
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
}
