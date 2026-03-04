//! Typed key structs for sled trees with compound keys.

use observation_tools_shared::ExecutionId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::PayloadId;

/// Key for the execution_observations tree: `{execution_id}:{observation_id}`
pub(crate) struct ExecutionObservationKey<'a> {
  pub execution_id: &'a ExecutionId,
  pub observation_id: &'a ObservationId,
}

impl ExecutionObservationKey<'_> {
  pub fn encode(&self) -> String {
    format!("{}:{}", self.execution_id, self.observation_id)
  }

  /// Prefix for scanning all observations belonging to an execution.
  pub fn encode_prefix(execution_id: &ExecutionId) -> String {
    format!("{}:", execution_id)
  }
}

/// Key for the payloads tree: `{observation_id}:{payload_id}`
pub(crate) struct PayloadKey<'a> {
  pub observation_id: &'a ObservationId,
  pub payload_id: &'a PayloadId,
}

impl PayloadKey<'_> {
  pub fn encode(&self) -> String {
    format!("{}:{}", self.observation_id, self.payload_id.as_str())
  }

  /// Prefix for scanning all payloads belonging to an observation.
  pub fn encode_prefix(observation_id: &ObservationId) -> String {
    format!("{}:", observation_id)
  }

  /// Extract the payload_id portion from a full key given the prefix.
  pub fn parse_payload_id<'k>(key: &'k str, prefix: &str) -> Option<&'k str> {
    key.strip_prefix(prefix)
  }
}

/// Key for the group_children tree: `{execution_id}:{parent_id}:{child_id}`
pub(crate) struct GroupChildrenKey<'a> {
  pub execution_id: &'a ExecutionId,
  pub parent_id: &'a str,
  pub child_id: &'a str,
}

impl GroupChildrenKey<'_> {
  pub fn encode(&self) -> String {
    format!("{}:{}:{}", self.execution_id, self.parent_id, self.child_id)
  }

  /// Prefix for scanning all children of a parent within an execution.
  pub fn encode_prefix(execution_id: &ExecutionId, parent_id: &str) -> String {
    format!("{}:{}:", execution_id, parent_id)
  }
}
