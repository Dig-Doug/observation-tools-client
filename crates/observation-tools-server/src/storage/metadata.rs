//! Metadata storage for executions and observations

use super::proto::StoredInlinePayload;
use super::proto::StoredObservation;
use super::proto::StoredPayloadMeta;
use super::ObservationWithPayloads;
use super::PayloadData;
use super::StorageError;
use super::StorageResult;
use super::StoredPayload;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use prost::Message;
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
  async fn list_observations(
    &self,
    execution_id: ExecutionId,
    limit: Option<usize>,
    offset: Option<usize>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<Vec<ObservationWithPayloads>>;

  /// Count total number of observations for an execution (with optional type
  /// filter)
  async fn count_observations(
    &self,
    execution_id: ExecutionId,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<usize>;
}

/// Sled-based metadata storage implementation
pub struct SledStorage {
  db: sled::Db,
}

/// Separator bytes for sled key structure:
/// {obs_id_16bytes}\x00\x00 -> metadata
/// {obs_id_16bytes}\x00\x01{payload_id_16bytes} -> inline payload
const KEY_SEP: u8 = 0x00;
const KEY_METADATA_SUFFIX: u8 = 0x00;
const KEY_PAYLOAD_SUFFIX: u8 = 0x01;

fn obs_id_bytes(obs_id: &ObservationId) -> Vec<u8> {
  // Use the string representation as bytes for the key
  obs_id.to_string().into_bytes()
}

fn metadata_key(obs_id: &ObservationId) -> Vec<u8> {
  let mut key = obs_id_bytes(obs_id);
  key.push(KEY_SEP);
  key.push(KEY_METADATA_SUFFIX);
  key
}

fn inline_payload_key(
  obs_id: &ObservationId,
  payload_id: &observation_tools_shared::PayloadId,
) -> Vec<u8> {
  let mut key = obs_id_bytes(obs_id);
  key.push(KEY_SEP);
  key.push(KEY_PAYLOAD_SUFFIX);
  key.extend_from_slice(payload_id.to_string().as_bytes());
  key
}

fn obs_prefix(obs_id: &ObservationId) -> Vec<u8> {
  let mut prefix = obs_id_bytes(obs_id);
  prefix.push(KEY_SEP);
  prefix
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
        Ok(StoredPayload {
          id: pm.payload_id()?,
          name: pm.name.clone(),
          mime_type: pm.mime_type.clone(),
          size: pm.size as usize,
          data: PayloadData::Blob,
        })
      })
      .collect::<StorageResult<Vec<_>>>()?;

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

    for item in obs_tree.scan_prefix(&prefix) {
      let (key, value) = item?;
      // Check if this is the metadata key or a payload key
      let suffix_start = prefix.len();
      if key.len() == suffix_start + 1 && key[suffix_start] == KEY_METADATA_SUFFIX {
        stored_obs = Some(StoredObservation::decode(value.as_ref())?);
      } else if key.len() > suffix_start + 1 && key[suffix_start] == KEY_PAYLOAD_SUFFIX {
        let payload_id_str =
          String::from_utf8(key[suffix_start + 1..].to_vec())
            .map_err(|e| StorageError::Internal(format!("Invalid payload key: {}", e)))?;
        let stored_payload = StoredInlinePayload::decode(value.as_ref())?;
        inline_data.insert(payload_id_str, stored_payload.data);
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
        Ok(StoredPayload {
          id: pm.payload_id()?,
          name: pm.name.clone(),
          mime_type: pm.mime_type.clone(),
          size: pm.size as usize,
          data,
        })
      })
      .collect::<StorageResult<Vec<_>>>()?;

    Ok(ObservationWithPayloads {
      observation,
      payloads,
    })
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
    for obs_with_payloads in observations {
      let obs = &obs_with_payloads.observation;
      let obs_id = obs.id;

      // Build the stored observation with payload manifest
      let mut stored = StoredObservation::from_observation(obs);
      for payload in &obs_with_payloads.payloads {
        stored.payload_manifest.push(StoredPayloadMeta {
          payload_id: payload.id.to_string(),
          name: payload.name.clone(),
          mime_type: payload.mime_type.clone(),
          size: payload.size as u64,
          is_blob: matches!(payload.data, PayloadData::Blob),
        });
      }

      // Store the metadata key
      let key = metadata_key(&obs_id);
      let value = stored.encode_to_vec();
      obs_tree.insert(key, value)?;

      // Store inline payloads
      for payload in &obs_with_payloads.payloads {
        if let PayloadData::Inline(ref data) = payload.data {
          let pkey = inline_payload_key(&obs_id, &payload.id);
          let stored_payload = StoredInlinePayload { data: data.clone() };
          obs_tree.insert(pkey, stored_payload.encode_to_vec())?;
        }
      }

      // Update the execution->observations index
      let exec_key = format!("{}:{}", obs.execution_id, obs.id);
      trace!("Storing execution-observation index: {}", exec_key);
      exec_obs_tree.insert(exec_key.as_bytes(), obs.id.to_string().as_bytes())?;
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
            .get(&key)
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
            .get(&key)
            .ok()
            .flatten()
            .and_then(|v| self.decode_metadata_only(&v).ok())
        })
      })
      .filter(|obs| observation_type.map_or(true, |t| obs.observation.observation_type == t))
      .count();
    Ok(count)
  }
}
