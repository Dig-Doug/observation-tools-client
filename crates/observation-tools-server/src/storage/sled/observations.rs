//! Observation storage methods for SledStorage

use super::keys::ExecutionObservationKey;
use super::keys::GroupChildrenKey;
use super::metadata_key;
use super::SledStorage;
use super::ROOT_SENTINEL;
use crate::storage::proto::StoredGroupChild;
use crate::storage::proto::StoredObservation;
use crate::storage::ObservationPage;
use crate::storage::ObservationStorage;
use crate::storage::PaginationInfo;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use crate::storage::PAGE_SIZE;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::Observation;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use prost::Message;
use tracing::trace;

#[async_trait::async_trait]
impl ObservationStorage for SledStorage {
  async fn store_observations(
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

  async fn get_observation(
    &self,
    id: ObservationId,
  ) -> StorageResult<Observation> {
    let obs_tree = self.observations_tree()?;
    self
      .decode_observation(&obs_tree, &id)?
      .ok_or_else(|| StorageError::NotFound(format!("Observation {} not found", id)))
  }

  async fn get_observations(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<ObservationPage> {
    let obs_tree = self.observations_tree()?;
    let exec_obs_tree = self.execution_observations_tree()?;
    let prefix = ExecutionObservationKey::encode_prefix(&execution_id);

    let start_key = match page_token {
      Some(ref token) => format!("{}{}\x00", prefix, token),
      None => prefix.clone(),
    };

    let mut observations = Vec::new();

    for item in exec_obs_tree
      .range(start_key.as_bytes()..)
      .take_while({
        let prefix = prefix.clone();
        move |item| {
          item
            .as_ref()
            .map(|(k, _)| k.starts_with(prefix.as_bytes()))
            .unwrap_or(false)
        }
      })
    {
      let (_key, obs_id_bytes) = item?;
      let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
        .map_err(|e| StorageError::Internal(format!("Invalid obs ID encoding: {}", e)))?;
      let obs_id = ObservationId::parse(&obs_id_str)
        .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;
      let obs = self.decode_observation(&obs_tree, &obs_id)?;
      if let Some(obs) = obs {
        if observation_type.map_or(true, |t| obs.observation_type == t) {
          observations.push(obs);
          if observations.len() > PAGE_SIZE {
            break;
          }
        }
      }
    }

    let next_page_token = if observations.len() > PAGE_SIZE {
      observations.pop();
      observations.last().map(|o| o.id.to_string())
    } else {
      None
    };

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
}

impl SledStorage {
  fn decode_observation(
    &self,
    obs_tree: &sled::Tree,
    id: &ObservationId,
  ) -> StorageResult<Option<Observation>> {
    let key = metadata_key(id);
    match obs_tree.get(key.as_bytes())? {
      Some(value) => {
        let stored = StoredObservation::decode(value.as_ref())?;
        Ok(Some(stored.to_observation()?))
      }
      None => Ok(None),
    }
  }

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
  use crate::storage::{ExecutionStorage, ObservationStorage};
  use observation_tools_shared::ObservationId;

  #[tokio::test]
  async fn test_store_and_get_observation() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs = make_observation(exec.id, "my-obs");
    let obs_id = obs.id;
    storage.store_observations(vec![obs]).await?;

    let result = storage.get_observation(obs_id).await?;
    assert_eq!(result.name, "my-obs");
    assert_eq!(result.execution_id, exec.id);
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observation_not_found() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();

    let result = storage.get_observation(ObservationId::new()).await;
    assert!(result.is_err());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observations_empty() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let page = storage.get_observations(exec.id, None, None).await?;
    assert_eq!(page.observations.len(), 0);
    assert!(page.pagination.next_page_token.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observations_returns_all() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs: Vec<_> = (0..5)
      .map(|i| make_observation(exec.id, &format!("obs-{}", i)))
      .collect();
    storage.store_observations(obs).await?;

    let page = storage.get_observations(exec.id, None, None).await?;
    assert_eq!(page.observations.len(), 5);
    assert!(page.pagination.next_page_token.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observations_pagination() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let count = crate::storage::PAGE_SIZE + 5;
    let obs: Vec<_> = (0..count)
      .map(|i| make_observation(exec.id, &format!("obs-{}", i)))
      .collect();
    storage.store_observations(obs).await?;

    let page1 = storage.get_observations(exec.id, None, None).await?;
    assert_eq!(page1.observations.len(), crate::storage::PAGE_SIZE);
    assert!(page1.pagination.next_page_token.is_some());
    assert!(page1.pagination.previous_page_token.is_none());

    let page2 = storage
      .get_observations(exec.id, page1.pagination.next_page_token, None)
      .await?;
    assert_eq!(page2.observations.len(), 5);
    assert!(page2.pagination.next_page_token.is_none());
    assert!(page2.pagination.previous_page_token.is_some());

    // No overlap between pages
    let page1_ids: std::collections::HashSet<_> =
      page1.observations.iter().map(|o| o.id).collect();
    for o in &page2.observations {
      assert!(!page1_ids.contains(&o.id));
    }
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observations_isolates_executions() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec1 = make_execution();
    let exec2 = make_execution();
    storage.store_execution(&exec1).await?;
    storage.store_execution(&exec2).await?;

    storage
      .store_observations(vec![
        make_observation(exec1.id, "exec1-obs"),
        make_observation(exec2.id, "exec2-obs"),
      ])
      .await?;

    let page1 = storage.get_observations(exec1.id, None, None).await?;
    assert_eq!(page1.observations.len(), 1);
    assert_eq!(page1.observations[0].name, "exec1-obs");

    let page2 = storage.get_observations(exec2.id, None, None).await?;
    assert_eq!(page2.observations.len(), 1);
    assert_eq!(page2.observations[0].name, "exec2-obs");
    Ok(())
  }
}
