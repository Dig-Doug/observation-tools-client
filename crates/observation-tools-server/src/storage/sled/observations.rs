//! Observation storage methods for SledStorage

use super::keys::ExecutionObservationKey;
use super::keys::GroupChildrenKey;
use super::metadata_key;
use super::SledStorage;
use super::ROOT_SENTINEL;
use crate::storage::proto::StoredGroupChild;
use crate::storage::proto::StoredObservation;
use crate::storage::ObservationPage;
use crate::storage::ObservationWithPayloads;
use crate::storage::PaginationInfo;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use crate::storage::PAGE_SIZE;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use prost::Message;
use tracing::trace;

impl SledStorage {
  pub(super) fn store_observations_impl(
    &self,
    observations: Vec<Observation>,
  ) -> StorageResult<()> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let gc_tree = self.group_children_tree()?;
    let gi_tree = self.group_index_tree()?;

    for obs in &observations {
      let obs_id = obs.id;

      // Store the observation metadata
      let stored = StoredObservation::from_observation(obs);
      let key = metadata_key(&obs_id);
      obs_tree.insert(key.as_bytes(), stored.encode_to_vec())?;

      // Update the execution->observations index
      let exec_key = ExecutionObservationKey {
        execution_id: &obs.execution_id,
        observation_id: &obs.id,
      }
      .encode();
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

  pub(super) fn get_observation_impl(
    &self,
    id: ObservationId,
  ) -> StorageResult<Observation> {
    let obs_tree = self.observations_tree()?;
    let key = metadata_key(&id);
    let meta_value = obs_tree
      .get(key.as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Observation {} not found", id)))?;
    let stored = StoredObservation::decode(meta_value.as_ref())?;
    stored.to_observation()
  }

  pub(super) fn get_observations_impl(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<ObservationPage> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = ExecutionObservationKey::encode_prefix(&execution_id);

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
        let obs = self.decode_metadata_only(&obs_id, &v)?;
        if observation_type.map_or(true, |t| obs.observation.observation_type == t) {
          count += 1;
          if count <= PAGE_SIZE {
            observations.push(obs);
          } else {
            next_page_token = observations.last().map(|o| o.observation.id.to_string());
            break;
          }
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

  pub(super) fn get_observation_by_group_id_impl(
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
    let value = obs_tree
      .get(metadata_key(&obs_id).as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Observation {} not found", obs_id)))?;
    self.decode_metadata_only(&obs_id, &value)
  }
}

impl SledStorage {
  pub(super) fn index_group_child(
    &self,
    gc_tree: &sled::Tree,
    execution_id: &ExecutionId,
    obs: &observation_tools_shared::Observation,
  ) -> StorageResult<()> {
    let child_id = obs.id.to_string();
    let is_group = obs.observation_type == ObservationType::Group;

    if is_group {
      let parent = obs
        .parent_group_id
        .as_ref()
        .map_or(ROOT_SENTINEL.to_string(), |g| g.as_str().to_string());
      let key = GroupChildrenKey {
        execution_id,
        parent_id: &parent,
        child_id: &child_id,
      }
      .encode();
      let value = StoredGroupChild {
        child_id,
        is_group: true,
      };
      gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
    } else if obs.group_ids.is_empty() {
      let key = GroupChildrenKey {
        execution_id,
        parent_id: ROOT_SENTINEL,
        child_id: &child_id,
      }
      .encode();
      let value = StoredGroupChild {
        child_id,
        is_group: false,
      };
      gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
    } else {
      for group_id in &obs.group_ids {
        let key = GroupChildrenKey {
          execution_id,
          parent_id: group_id.as_str(),
          child_id: &child_id,
        }
        .encode();
        let value = StoredGroupChild {
          child_id: child_id.clone(),
          is_group: false,
        };
        gc_tree.insert(key.as_bytes(), value.encode_to_vec())?;
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::super::test_helpers::*;
  use crate::storage::MetadataStorage;

  #[tokio::test]
  async fn test_get_observations_pagination() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let mut obs = Vec::new();
    for i in 0..5 {
      obs.push(make_observation(exec.id, &format!("obs-{}", i)));
    }
    storage.store_observations(obs).await.expect("store obs");

    let page = storage
      .get_observations(exec.id, None, None)
      .await
      .expect("get page");
    assert_eq!(page.observations.len(), 5);
    assert!(page.pagination.next_page_token.is_none());
  }
}
