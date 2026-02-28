//! Metadata storage for executions and observations

use super::proto::StoredGroupChild;
use super::proto::StoredInlinePayload;
use super::proto::StoredObservation;
use super::proto::StoredPayloadMeta;
use super::Group;
use super::GroupDirectDescendantsPage;
use super::GroupTree;
use super::GroupTreeNode;
use super::ObservationPage;
use super::ObservationPayloadPage;
use super::ObservationWithPayloads;
use super::PaginationInfo;
use super::PayloadData;
use super::StorageError;
use super::StorageResult;
use super::StoredPayload;
use super::PAGE_SIZE;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::PayloadId;
use prost::Message;
use std::collections::VecDeque;
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

  /// Store multiple observations with their payloads in a batch
  async fn store_observations(
    &self,
    observations: Vec<ObservationWithPayloads>,
  ) -> StorageResult<()>;

  /// Get a single observation with all inline payload data (via prefix scan)
  async fn get_observation(
    &self,
    id: ObservationId,
  ) -> StorageResult<ObservationWithPayloads>;

  /// List observations for an execution (with optional pagination and type filter).
  /// Returns observations with all payloads as PayloadData::Blob (metadata only).
  /// Retained for UI backward compatibility - prefer get_observations for new code.
  async fn list_observations(
    &self,
    execution_id: ExecutionId,
    limit: Option<usize>,
    offset: Option<usize>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<Vec<ObservationWithPayloads>>;

  /// Count total number of observations for an execution (with optional type filter).
  /// Retained for UI backward compatibility.
  async fn count_observations(
    &self,
    execution_id: ExecutionId,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<usize>;

  /// Paginated observations sorted by creation time (UUIDv7 order).
  /// Uses cursor-based pagination with page tokens.
  async fn get_observations(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPage>;

  /// Paginated payload retrieval for observation detail panel.
  /// Uses payload_id as cursor.
  async fn get_payloads(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage>;

  /// Direct children of a group (or root if group_id is None).
  /// Uses cursor-based pagination.
  async fn get_direct_descendants_page(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    page_token: Option<String>,
  ) -> StorageResult<GroupDirectDescendantsPage>;

  /// Look up a group observation by its GroupId.
  /// Returns the observation that represents this group.
  async fn get_observation_by_group_id(
    &self,
    group_id: GroupId,
  ) -> StorageResult<ObservationWithPayloads>;

  /// BFS expansion of group tree up to max_nodes.
  /// Returns Tree if first level fits, List if first level exceeds max_nodes.
  async fn get_group_tree_bfs(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    max_nodes: usize,
  ) -> StorageResult<GroupTree>;
}

/// Sled-based metadata storage implementation
pub struct SledStorage {
  db: sled::Db,
}

/// String-based key format for readability:
/// "{obs_id}:meta" -> observation metadata (protobuf)
/// "{obs_id}:payload:{payload_id}" -> inline payload data (protobuf)
const KEY_META_SUFFIX: &str = ":meta";
const KEY_PAYLOAD_INFIX: &str = ":payload:";

/// Sentinel value for root-level items in the group_children tree
const ROOT_SENTINEL: &str = "_ROOT_";

fn metadata_key(obs_id: &ObservationId) -> String {
  format!("{}{}", obs_id, KEY_META_SUFFIX)
}

fn inline_payload_key(obs_id: &ObservationId, payload_id: &PayloadId) -> String {
  format!("{}{}{}", obs_id, KEY_PAYLOAD_INFIX, payload_id.as_str())
}

fn obs_prefix(obs_id: &ObservationId) -> String {
  format!("{}:", obs_id)
}

/// Build a group_children key: {execution_id}:{parent_id}:{child_id}
fn group_children_key(execution_id: &ExecutionId, parent_id: &str, child_id: &str) -> String {
  format!("{}:{}:{}", execution_id, parent_id, child_id)
}

/// Build a group_children prefix for scanning all children of a parent
fn group_children_prefix(execution_id: &ExecutionId, parent_id: &str) -> String {
  format!("{}:{}:", execution_id, parent_id)
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

  /// Get the group_children index tree
  fn group_children_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("group_children")?)
  }

  /// Get the group_index tree (group_id → observation_id)
  fn group_index_tree(&self) -> StorageResult<sled::Tree> {
    Ok(self.db.open_tree("group_index")?)
  }

  /// Decode a stored observation from a metadata key's value, returning
  /// observation with all payloads marked as Blob
  fn decode_metadata_only(
    &self,
    value: &[u8],
  ) -> StorageResult<ObservationWithPayloads> {
    let stored = StoredObservation::decode(value)?;
    let observation = stored.to_observation()?;
    let payloads = stored
      .payload_manifest
      .iter()
      .map(|pm| {
        StoredPayload {
          id: pm.to_payload_id(),
          name: pm.name.clone(),
          mime_type: pm.mime_type.clone(),
          size: pm.size as usize,
          data: PayloadData::Blob,
        }
      })
      .collect();

    Ok(ObservationWithPayloads {
      observation,
      payloads,
    })
  }

  /// Decode a stored observation via prefix scan, returning observation with
  /// inline payload data where available
  fn decode_with_inline_payloads(
    &self,
    obs_tree: &sled::Tree,
    obs_id: &ObservationId,
  ) -> StorageResult<ObservationWithPayloads> {
    let prefix = obs_prefix(obs_id);
    let mut stored_obs: Option<StoredObservation> = None;
    let mut inline_data: std::collections::HashMap<String, Vec<u8>> =
      std::collections::HashMap::new();

    for item in obs_tree.scan_prefix(prefix.as_bytes()) {
      let (key, value) = item?;
      let key_str = String::from_utf8(key.to_vec())
        .map_err(|e| StorageError::Internal(format!("Invalid key encoding: {}", e)))?;

      if key_str.ends_with(KEY_META_SUFFIX) {
        stored_obs = Some(StoredObservation::decode(value.as_ref())?);
      } else if let Some(payload_id_str) = key_str
        .strip_prefix(&format!("{}{}", obs_id, KEY_PAYLOAD_INFIX))
      {
        let stored_payload = StoredInlinePayload::decode(value.as_ref())?;
        inline_data.insert(payload_id_str.to_string(), stored_payload.data);
      }
    }

    let stored = stored_obs.ok_or_else(|| {
      StorageError::NotFound(format!("Observation {} not found", obs_id))
    })?;

    let observation = stored.to_observation()?;
    let payloads = stored
      .payload_manifest
      .iter()
      .map(|pm| {
        let data = if pm.is_blob {
          PayloadData::Blob
        } else if let Some(inline) = inline_data.remove(&pm.payload_id) {
          PayloadData::Inline(inline)
        } else {
          PayloadData::Blob
        };
        StoredPayload {
          id: pm.to_payload_id(),
          name: pm.name.clone(),
          mime_type: pm.mime_type.clone(),
          size: pm.size as usize,
          data,
        }
      })
      .collect();

    Ok(ObservationWithPayloads {
      observation,
      payloads,
    })
  }

  /// Index a single observation into the group_children tree
  fn index_group_child(
    &self,
    gc_tree: &sled::Tree,
    execution_id: &ExecutionId,
    obs: &observation_tools_shared::Observation,
  ) -> StorageResult<()> {
    let child_id = obs.id.to_string();
    let is_group = obs.observation_type == ObservationType::Group;

    if is_group {
      // Group observations: index under parent_group_id (or ROOT if none)
      let parent = obs
        .parent_group_id
        .as_ref()
        .map_or(ROOT_SENTINEL.to_string(), |g| g.as_str().to_string());
      let key = group_children_key(execution_id, &parent, &child_id);
      let value = StoredGroupChild {
        child_id,
        is_group: true,
      };
      gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
    } else if obs.group_ids.is_empty() {
      // Non-group observations with no group membership: index under ROOT
      let key = group_children_key(execution_id, ROOT_SENTINEL, &child_id);
      let value = StoredGroupChild {
        child_id,
        is_group: false,
      };
      gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
    } else {
      // Non-group observations: index under each group_id
      for group_id in &obs.group_ids {
        let key = group_children_key(execution_id, group_id.as_str(), &child_id);
        let value = StoredGroupChild {
          child_id: child_id.clone(),
          is_group: false,
        };
        gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
      }
    }

    Ok(())
  }

  /// Walk up the parent chain to compute group ancestors (oldest ancestor first)
  fn compute_group_ancestors(
    &self,
    obs_tree: &sled::Tree,
    group_obs: &ObservationWithPayloads,
  ) -> StorageResult<Vec<GroupId>> {
    let gi_tree = self.group_index_tree()?;
    let mut ancestors = Vec::new();
    let mut current_parent = group_obs.observation.parent_group_id.clone();

    while let Some(ref parent_id) = current_parent {
      // Prevent infinite loops
      if ancestors.iter().any(|a: &GroupId| a.as_str() == parent_id.as_str()) {
        break;
      }
      ancestors.push(parent_id.clone());

      // Look up the observation ID for this group via the group_index
      let obs_id = match gi_tree.get(parent_id.as_str().as_bytes())? {
        Some(obs_id_bytes) => {
          let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
            .map_err(|e| StorageError::Internal(format!("Invalid obs ID encoding: {}", e)))?;
          match ObservationId::parse(&obs_id_str) {
            Ok(id) => id,
            Err(_) => break,
          }
        }
        None => break,
      };

      let key = metadata_key(&obs_id);
      match obs_tree.get(key.as_bytes())? {
        Some(value) => {
          let stored = StoredObservation::decode(value.as_ref())?;
          let obs = stored.to_observation()?;
          current_parent = obs.parent_group_id;
        }
        None => break,
      }
    }

    // Reverse so oldest ancestor is first
    ancestors.reverse();
    Ok(ancestors)
  }

  /// Scan group_children for direct descendants of a parent, with cursor pagination
  fn scan_direct_descendants(
    &self,
    gc_tree: &sled::Tree,
    obs_tree: &sled::Tree,
    execution_id: &ExecutionId,
    parent_id: &str,
    page_token: Option<&str>,
    limit: usize,
  ) -> StorageResult<(Vec<(StoredGroupChild, ObservationWithPayloads)>, Option<String>)> {
    let prefix = group_children_prefix(execution_id, parent_id);

    let iter: Box<dyn Iterator<Item = sled::Result<(sled::IVec, sled::IVec)>>> =
      if let Some(token) = page_token {
        // Start scanning from one past the last seen key
        let start_key = format!("{}{}\x00", prefix, token);
        Box::new(
          gc_tree
            .range(start_key.as_bytes()..)
            .take_while({
              let prefix = prefix.clone();
              move |item| {
                item
                  .as_ref()
                  .map(|(k, _)| k.starts_with(prefix.as_bytes()))
                  .unwrap_or(false)
              }
            }),
        )
      } else {
        Box::new(gc_tree.scan_prefix(prefix.as_bytes()))
      };

    let mut results = Vec::new();
    let mut count = 0;

    for item in iter {
      let (_key, value) = item?;
      let child = StoredGroupChild::decode(value.as_ref())?;

      let child_obs_id = ObservationId::parse(&child.child_id)
        .map_err(|e| StorageError::Internal(format!("Invalid child ID: {}", e)))?;
      let key = metadata_key(&child_obs_id);
      let obs_value = obs_tree.get(key.as_bytes())?;
      if let Some(v) = obs_value {
        let obs = self.decode_metadata_only(&v)?;
        count += 1;
        if count <= limit {
          results.push((child, obs));
        } else {
          // We have one extra, so there's a next page
          let last_id = results.last().map(|(c, _)| c.child_id.clone());
          return Ok((results, last_id));
        }
      }
    }

    Ok((results, None))
  }

  /// Build a GroupTreeNode from a StoredGroupChild and its observation data
  fn build_tree_node(
    &self,
    child: &StoredGroupChild,
    obs: ObservationWithPayloads,
    obs_tree: &sled::Tree,
  ) -> StorageResult<GroupTreeNode> {
    if child.is_group {
      let ancestors = self.compute_group_ancestors(obs_tree, &obs)?;
      let content = GroupDirectDescendantsPage {
        descendants: Vec::new(),
        pagination: PaginationInfo {
          item_count: 0,
          previous_page_token: None,
          next_page_token: None,
        },
      };
      Ok(GroupTreeNode::Group(Group {
        metadata: obs,
        group_ancestors: ancestors,
        content,
      }))
    } else {
      Ok(GroupTreeNode::Observation(obs))
    }
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

  async fn store_observations(
    &self,
    observations: Vec<ObservationWithPayloads>,
  ) -> StorageResult<()> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let gc_tree = self.group_children_tree()?;
    let gi_tree = self.group_index_tree()?;

    for obs_with_payloads in observations {
      let obs = &obs_with_payloads.observation;
      let obs_id = obs.id;

      // Build the stored observation with payload manifest
      let mut stored = StoredObservation::from_observation(obs);
      for payload in &obs_with_payloads.payloads {
        stored.payload_manifest.push(StoredPayloadMeta {
          payload_id: payload.id.as_str().to_string(),
          name: payload.name.clone(),
          mime_type: payload.mime_type.clone(),
          size: payload.size as u64,
          is_blob: matches!(payload.data, PayloadData::Blob),
        });
      }

      // Store the metadata key
      let key = metadata_key(&obs_id);
      let value = stored.encode_to_vec();
      obs_tree.insert(key.as_bytes(), value)?;

      // Store inline payloads
      for payload in &obs_with_payloads.payloads {
        if let PayloadData::Inline(ref data) = payload.data {
          let pkey = inline_payload_key(&obs_id, &payload.id);
          let stored_payload = StoredInlinePayload { data: data.clone() };
          obs_tree.insert(pkey.as_bytes(), stored_payload.encode_to_vec())?;
        }
      }

      // Update the execution->observations index
      let exec_key = format!("{}:{}", obs.execution_id, obs.id);
      trace!("Storing execution-observation index: {}", exec_key);
      exec_obs_tree.insert(exec_key.as_bytes(), obs.id.to_string().as_bytes())?;

      // Index group children relationships
      self.index_group_child(&gc_tree, &obs.execution_id, obs)?;

      // Index group_id → observation_id for group observations
      if obs.observation_type == ObservationType::Group {
        for group_id in &obs.group_ids {
          gi_tree.insert(
            group_id.as_str().as_bytes(),
            obs_id.to_string().as_bytes(),
          )?;
        }
      }
    }
    Ok(())
  }

  async fn get_observation(
    &self,
    id: ObservationId,
  ) -> StorageResult<ObservationWithPayloads> {
    let obs_tree = self.observations_tree()?;
    self.decode_with_inline_payloads(&obs_tree, &id)
  }

  async fn list_observations(
    &self,
    execution_id: ExecutionId,
    limit: Option<usize>,
    offset: Option<usize>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<Vec<ObservationWithPayloads>> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = format!("{}:", execution_id);
    let observations: Vec<ObservationWithPayloads> = exec_obs_tree
      .scan_prefix(prefix.as_bytes())
      .values()
      .filter_map(|result| {
        result.ok().and_then(|obs_id_bytes| {
          let obs_id_str = String::from_utf8(obs_id_bytes.to_vec()).ok()?;
          let obs_id = ObservationId::parse(&obs_id_str).ok()?;
          let key = metadata_key(&obs_id);
          obs_tree
            .get(key.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| self.decode_metadata_only(&v).ok())
        })
      })
      .filter(|obs| observation_type.map_or(true, |t| obs.observation.observation_type == t))
      .skip(offset.unwrap_or(0))
      .take(limit.unwrap_or(100))
      .collect();
    Ok(observations)
  }

  async fn count_observations(
    &self,
    execution_id: ExecutionId,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<usize> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = format!("{}:", execution_id);
    let count = exec_obs_tree
      .scan_prefix(prefix.as_bytes())
      .values()
      .filter_map(|result| {
        result.ok().and_then(|obs_id_bytes| {
          let obs_id_str = String::from_utf8(obs_id_bytes.to_vec()).ok()?;
          let obs_id = ObservationId::parse(&obs_id_str).ok()?;
          let key = metadata_key(&obs_id);
          obs_tree
            .get(key.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| self.decode_metadata_only(&v).ok())
        })
      })
      .filter(|obs| observation_type.map_or(true, |t| obs.observation.observation_type == t))
      .count();
    Ok(count)
  }

  async fn get_observations(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPage> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = format!("{}:", execution_id);

    let iter: Box<dyn Iterator<Item = sled::Result<(sled::IVec, sled::IVec)>>> =
      if let Some(ref token) = page_token {
        let start_key = format!("{}{}\x00", prefix, token);
        Box::new(
          exec_obs_tree
            .range(start_key.as_bytes()..)
            .take_while({
              let prefix = prefix.clone();
              move |item| {
                item
                  .as_ref()
                  .map(|(k, _)| k.starts_with(prefix.as_bytes()))
                  .unwrap_or(false)
              }
            }),
        )
      } else {
        Box::new(exec_obs_tree.scan_prefix(prefix.as_bytes()))
      };

    let mut observations = Vec::new();
    let mut next_page_token = None;
    let mut count = 0;

    for item in iter {
      let (_key, obs_id_bytes) = item?;
      let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
        .map_err(|e| StorageError::Internal(format!("Invalid obs ID encoding: {}", e)))?;
      let obs_id = ObservationId::parse(&obs_id_str)
        .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;
      let key = metadata_key(&obs_id);
      if let Some(v) = obs_tree.get(key.as_bytes())? {
        let obs = self.decode_metadata_only(&v)?;
        count += 1;
        if count <= PAGE_SIZE {
          observations.push(obs);
        } else {
          // We have one extra item, so there's a next page
          next_page_token = observations.last().map(|o| o.observation.id.to_string());
          break;
        }
      }
    }

    let item_count = observations.len();
    Ok(ObservationPage {
      observations,
      pagination: PaginationInfo {
        item_count,
        previous_page_token: page_token,
        next_page_token,
      },
    })
  }

  async fn get_payloads(
    &self,
    _execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage> {
    let obs_tree = self.observations_tree()?;
    let obs = self.decode_with_inline_payloads(&obs_tree, &observation_id)?;

    // Filter payloads using cursor (payload_id)
    let mut payloads: Vec<StoredPayload> = if let Some(ref token) = page_token {
      obs
        .payloads
        .into_iter()
        .skip_while(|p| p.id.as_str() <= token.as_str())
        .collect()
    } else {
      obs.payloads
    };

    let next_page_token = if payloads.len() > PAGE_SIZE {
      payloads.truncate(PAGE_SIZE);
      payloads.last().map(|p| p.id.as_str().to_string())
    } else {
      None
    };

    let item_count = payloads.len();
    Ok(ObservationPayloadPage {
      payloads,
      pagination: PaginationInfo {
        item_count,
        previous_page_token: page_token,
        next_page_token,
      },
    })
  }

  async fn get_observation_by_group_id(
    &self,
    group_id: GroupId,
  ) -> StorageResult<ObservationWithPayloads> {
    let gi_tree = self.group_index_tree()?;
    let obs_id_bytes = gi_tree
      .get(group_id.as_str().as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Group {} not found", group_id.as_str())))?;
    let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID encoding: {}", e)))?;
    let obs_id = ObservationId::parse(&obs_id_str)
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;
    let obs_tree = self.observations_tree()?;
    self.decode_metadata_only(&obs_tree.get(metadata_key(&obs_id).as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Observation {} not found", obs_id)))?)
  }

  async fn get_direct_descendants_page(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    page_token: Option<String>,
  ) -> StorageResult<GroupDirectDescendantsPage> {
    let gc_tree = self.group_children_tree()?;
    let obs_tree = self.observations_tree()?;
    let parent_id = group_id
      .as_ref()
      .map_or(ROOT_SENTINEL, |g| g.as_str());

    let (results, next_page_token) = self.scan_direct_descendants(
      &gc_tree,
      &obs_tree,
      &execution_id,
      parent_id,
      page_token.as_deref(),
      PAGE_SIZE,
    )?;

    let mut descendants = Vec::new();
    for (child, obs) in results {
      let node = self.build_tree_node(&child, obs, &obs_tree)?;
      descendants.push(node);
    }

    let item_count = descendants.len();
    Ok(GroupDirectDescendantsPage {
      descendants,
      pagination: PaginationInfo {
        item_count,
        previous_page_token: page_token,
        next_page_token,
      },
    })
  }

  async fn get_group_tree_bfs(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    max_nodes: usize,
  ) -> StorageResult<GroupTree> {
    let gc_tree = self.group_children_tree()?;
    let obs_tree = self.observations_tree()?;
    let parent_id = group_id
      .as_ref()
      .map_or(ROOT_SENTINEL, |g| g.as_str());

    // First, get all root-level children (no limit for counting)
    let prefix = group_children_prefix(&execution_id, parent_id);
    let root_children: Vec<(StoredGroupChild, ObservationWithPayloads)> = gc_tree
      .scan_prefix(prefix.as_bytes())
      .filter_map(|item| {
        let (_key, value) = item.ok()?;
        let child = StoredGroupChild::decode(value.as_ref()).ok()?;
        let child_obs_id = ObservationId::parse(&child.child_id).ok()?;
        let key = metadata_key(&child_obs_id);
        let obs_value = obs_tree.get(key.as_bytes()).ok()??;
        let obs = self.decode_metadata_only(&obs_value).ok()?;
        Some((child, obs))
      })
      .collect();

    // If root level exceeds max_nodes, return as paginated list
    if root_children.len() > max_nodes {
      // Return first PAGE_SIZE with pagination
      let (page_results, next_token) = self.scan_direct_descendants(
        &gc_tree,
        &obs_tree,
        &execution_id,
        parent_id,
        None,
        PAGE_SIZE,
      )?;

      let mut descendants = Vec::new();
      for (child, obs) in page_results {
        let node = self.build_tree_node(&child, obs, &obs_tree)?;
        descendants.push(node);
      }

      let item_count = descendants.len();
      return Ok(GroupTree::List(GroupDirectDescendantsPage {
        descendants,
        pagination: PaginationInfo {
          item_count,
          previous_page_token: None,
          next_page_token: next_token,
        },
      }));
    }

    // BFS expansion
    let mut total_nodes = root_children.len();

    // Build initial root nodes
    let mut roots: Vec<GroupTreeNode> = Vec::new();
    // Queue of (parent_group_id, index_in_roots_path) for groups that need expansion
    let mut bfs_queue: VecDeque<(String, Vec<usize>)> = VecDeque::new();

    for (i, (child, obs)) in root_children.into_iter().enumerate() {
      if child.is_group {
        // Use group_ids[0] if available (the group's own GroupId),
        // otherwise fall back to child_id (observation ID) for backward compatibility
        let group_key = obs
          .observation
          .group_ids
          .first()
          .map(|g| g.as_str().to_string())
          .unwrap_or_else(|| child.child_id.clone());
        let ancestors = self.compute_group_ancestors(&obs_tree, &obs)?;
        let node = GroupTreeNode::Group(Group {
          metadata: obs,
          group_ancestors: ancestors,
          content: GroupDirectDescendantsPage {
            descendants: Vec::new(),
            pagination: PaginationInfo {
              item_count: 0,
              previous_page_token: None,
              next_page_token: None,
            },
          },
        });
        bfs_queue.push_back((group_key, vec![i]));
        roots.push(node);
      } else {
        roots.push(GroupTreeNode::Observation(obs));
      }
    }

    // BFS: expand group nodes level by level
    while let Some((group_child_id, path)) = bfs_queue.pop_front() {
      let child_prefix = group_children_prefix(&execution_id, &group_child_id);
      let children: Vec<(StoredGroupChild, ObservationWithPayloads)> = gc_tree
        .scan_prefix(child_prefix.as_bytes())
        .filter_map(|item| {
          let (_key, value) = item.ok()?;
          let child = StoredGroupChild::decode(value.as_ref()).ok()?;
          let child_obs_id = ObservationId::parse(&child.child_id).ok()?;
          let key = metadata_key(&child_obs_id);
          let obs_value = obs_tree.get(key.as_bytes()).ok()??;
          let obs = self.decode_metadata_only(&obs_value).ok()?;
          Some((child, obs))
        })
        .collect();

      // Check if adding these children would exceed max_nodes
      if total_nodes + children.len() > max_nodes {
        // Stop expanding - leave this group's content empty (or partially filled)
        // The caller can use get_direct_descendants_page to paginate
        continue;
      }

      total_nodes += children.len();

      let mut child_nodes = Vec::new();
      let mut new_queue_entries = Vec::new();

      for (j, (child, obs)) in children.into_iter().enumerate() {
        if child.is_group {
          let group_key = obs
            .observation
            .group_ids
            .first()
            .map(|g| g.as_str().to_string())
            .unwrap_or_else(|| child.child_id.clone());
          let ancestors = self.compute_group_ancestors(&obs_tree, &obs)?;
          let node = GroupTreeNode::Group(Group {
            metadata: obs,
            group_ancestors: ancestors,
            content: GroupDirectDescendantsPage {
              descendants: Vec::new(),
              pagination: PaginationInfo {
                item_count: 0,
                previous_page_token: None,
                next_page_token: None,
              },
            },
          });
          let mut child_path = path.clone();
          child_path.push(j);
          new_queue_entries.push((group_key, child_path));
          child_nodes.push(node);
        } else {
          child_nodes.push(GroupTreeNode::Observation(obs));
        }
      }

      // Set the children on the parent group node
      let child_count = child_nodes.len();
      set_group_content_at_path(
        &mut roots,
        &path,
        child_nodes,
        child_count,
      );

      for entry in new_queue_entries {
        bfs_queue.push_back(entry);
      }
    }

    Ok(GroupTree::Tree { roots })
  }
}

/// Navigate the tree by path indices and set a group's content
fn set_group_content_at_path(
  roots: &mut [GroupTreeNode],
  path: &[usize],
  children: Vec<GroupTreeNode>,
  child_count: usize,
) {
  if path.is_empty() {
    return;
  }

  let mut current = &mut roots[path[0]];
  for &idx in &path[1..] {
    current = match current {
      GroupTreeNode::Group(ref mut group) => {
        &mut group.content.descendants[idx]
      }
      GroupTreeNode::Observation(_) => return,
    };
  }

  if let GroupTreeNode::Group(ref mut group) = current {
    group.content.descendants = children;
    group.content.pagination.item_count = child_count;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use observation_tools_shared::Observation;
  use tempfile::TempDir;

  fn test_storage() -> (SledStorage, TempDir) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let storage = SledStorage::new(dir.path()).expect("failed to create storage");
    (storage, dir)
  }

  fn make_execution() -> Execution {
    Execution {
      id: ExecutionId::new(),
      name: "test".to_string(),
      metadata: Default::default(),
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    }
  }

  fn make_observation(exec_id: ExecutionId, name: &str) -> ObservationWithPayloads {
    ObservationWithPayloads {
      observation: Observation {
        id: ObservationId::new(),
        execution_id: exec_id,
        name: name.to_string(),
        observation_type: ObservationType::LogEntry,
        log_level: observation_tools_shared::LogLevel::Info,
        source: None,
        metadata: Default::default(),
        group_ids: Vec::new(),
        parent_group_id: None,
        parent_span_id: None,
        created_at: chrono::Utc::now(),
      },
      payloads: Vec::new(),
    }
  }

  fn make_group(exec_id: ExecutionId, name: &str, parent: Option<GroupId>) -> ObservationWithPayloads {
    let id = ObservationId::new();
    ObservationWithPayloads {
      observation: Observation {
        id,
        execution_id: exec_id,
        name: name.to_string(),
        observation_type: ObservationType::Group,
        log_level: observation_tools_shared::LogLevel::Info,
        source: None,
        metadata: Default::default(),
        group_ids: Vec::new(),
        parent_group_id: parent,
        parent_span_id: None,
        created_at: chrono::Utc::now(),
      },
      payloads: Vec::new(),
    }
  }

  fn make_obs_in_group(exec_id: ExecutionId, name: &str, group_id: GroupId) -> ObservationWithPayloads {
    ObservationWithPayloads {
      observation: Observation {
        id: ObservationId::new(),
        execution_id: exec_id,
        name: name.to_string(),
        observation_type: ObservationType::LogEntry,
        log_level: observation_tools_shared::LogLevel::Info,
        source: None,
        metadata: Default::default(),
        group_ids: vec![group_id],
        parent_group_id: None,
        parent_span_id: None,
        created_at: chrono::Utc::now(),
      },
      payloads: Vec::new(),
    }
  }

  #[tokio::test]
  async fn test_get_observations_pagination() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    // Store 5 observations
    let mut obs = Vec::new();
    for i in 0..5 {
      obs.push(make_observation(exec.id, &format!("obs-{}", i)));
    }
    storage.store_observations(obs).await.expect("store obs");

    // Get first page (all should fit in PAGE_SIZE)
    let page = storage.get_observations(exec.id, None).await.expect("get page");
    assert_eq!(page.observations.len(), 5);
    assert!(page.pagination.next_page_token.is_none());
  }

  #[tokio::test]
  async fn test_get_direct_descendants_root() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    // Store root-level observations
    let obs1 = make_observation(exec.id, "root-obs-1");
    let obs2 = make_observation(exec.id, "root-obs-2");
    storage
      .store_observations(vec![obs1, obs2])
      .await
      .expect("store obs");

    let page = storage
      .get_direct_descendants_page(exec.id, None, None)
      .await
      .expect("get descendants");
    assert_eq!(page.descendants.len(), 2);
  }

  #[tokio::test]
  async fn test_group_tree_bfs_simple() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    // Create a group with children
    let group = make_group(exec.id, "parent-group", None);
    let group_id = GroupId::from(group.observation.id.to_string());

    let child1 = make_obs_in_group(exec.id, "child-1", group_id.clone());
    let child2 = make_obs_in_group(exec.id, "child-2", group_id.clone());

    storage
      .store_observations(vec![group, child1, child2])
      .await
      .expect("store obs");

    let tree = storage
      .get_group_tree_bfs(exec.id, None, 100)
      .await
      .expect("get tree");

    match tree {
      GroupTree::Tree { roots } => {
        assert_eq!(roots.len(), 1);
        match &roots[0] {
          GroupTreeNode::Group(g) => {
            assert_eq!(g.metadata.observation.name, "parent-group");
            assert_eq!(g.content.descendants.len(), 2);
          }
          _ => panic!("expected group node"),
        }
      }
      GroupTree::List(_) => panic!("expected tree, got list"),
    }
  }

  #[tokio::test]
  async fn test_group_tree_bfs_exceeds_max() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    // Create many root-level observations
    let mut obs = Vec::new();
    for i in 0..10 {
      obs.push(make_observation(exec.id, &format!("obs-{}", i)));
    }
    storage.store_observations(obs).await.expect("store obs");

    // Request with max_nodes=5 (less than root count)
    let tree = storage
      .get_group_tree_bfs(exec.id, None, 5)
      .await
      .expect("get tree");

    match tree {
      GroupTree::List(page) => {
        assert!(!page.descendants.is_empty());
      }
      GroupTree::Tree { .. } => panic!("expected list, got tree"),
    }
  }

  #[tokio::test]
  async fn test_nested_groups() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    // Create nested groups: grandparent -> parent -> child obs
    let grandparent = make_group(exec.id, "grandparent", None);
    let gp_id = GroupId::from(grandparent.observation.id.to_string());

    let parent = make_group(exec.id, "parent", Some(gp_id.clone()));
    let p_id = GroupId::from(parent.observation.id.to_string());

    let child = make_obs_in_group(exec.id, "leaf", p_id.clone());

    storage
      .store_observations(vec![grandparent, parent, child])
      .await
      .expect("store obs");

    let tree = storage
      .get_group_tree_bfs(exec.id, None, 100)
      .await
      .expect("get tree");

    match tree {
      GroupTree::Tree { roots } => {
        assert_eq!(roots.len(), 1);
        match &roots[0] {
          GroupTreeNode::Group(g) => {
            assert_eq!(g.metadata.observation.name, "grandparent");
            assert_eq!(g.content.descendants.len(), 1);
            match &g.content.descendants[0] {
              GroupTreeNode::Group(inner) => {
                assert_eq!(inner.metadata.observation.name, "parent");
                assert_eq!(inner.group_ancestors.len(), 1);
                assert_eq!(inner.content.descendants.len(), 1);
              }
              _ => panic!("expected inner group"),
            }
          }
          _ => panic!("expected group"),
        }
      }
      GroupTree::List(_) => panic!("expected tree"),
    }
  }

  #[tokio::test]
  async fn test_get_payloads() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let mut obs = make_observation(exec.id, "with-payloads");
    obs.payloads = vec![
      StoredPayload {
        id: PayloadId::new(),
        name: "payload-1".to_string(),
        mime_type: "text/plain".to_string(),
        size: 5,
        data: PayloadData::Inline(b"hello".to_vec()),
      },
      StoredPayload {
        id: PayloadId::new(),
        name: "payload-2".to_string(),
        mime_type: "text/plain".to_string(),
        size: 5,
        data: PayloadData::Inline(b"world".to_vec()),
      },
    ];
    let obs_id = obs.observation.id;
    storage
      .store_observations(vec![obs])
      .await
      .expect("store obs");

    let page = storage
      .get_payloads(exec.id, obs_id, None)
      .await
      .expect("get payloads");
    assert_eq!(page.payloads.len(), 2);
  }
}
