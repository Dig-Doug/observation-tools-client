use super::keys::GroupChildrenKey;
use super::metadata_key;
use super::SledStorage;
use super::ROOT_SENTINEL;
use crate::storage::proto::StoredGroupChild;
use crate::storage::proto::StoredObservation;
use crate::storage::root_group_id;
use crate::storage::Group;
use crate::storage::GroupDirectDescendantsPage;
use crate::storage::GroupMembershipOptions;
use crate::storage::GroupStorage;
use crate::storage::GroupTreeNode;
use crate::storage::PaginationInfo;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use prost::Message;
use std::collections::HashMap;

#[async_trait::async_trait]
impl GroupStorage for SledStorage {
  async fn get_direct_descendants_page(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    page_token: Option<String>,
    page_size: usize,
  ) -> StorageResult<GroupDirectDescendantsPage> {
    let gc_tree = self.group_children_tree()?;
    let obs_tree = self.observations_tree()?;
    let parent_id = group_id.as_ref().map_or(ROOT_SENTINEL, |g| g.as_str());

    let (results, next_page_token) = self.scan_direct_descendants(
      &gc_tree,
      &obs_tree,
      &execution_id,
      parent_id,
      page_token.as_deref(),
      page_size,
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

  async fn get_observation_by_group_id(
    &self,
    group_id: GroupId,
  ) -> StorageResult<observation_tools_shared::Observation> {
    self
      .get_group(&group_id)?
      .ok_or_else(|| StorageError::NotFound(format!("Group {} not found", group_id.as_str())))
  }

  async fn get_descendants(
    &self,
    execution_id: ExecutionId,
    options: GroupMembershipOptions,
  ) -> StorageResult<HashMap<GroupId, GroupDirectDescendantsPage>> {
    let root_children = self
      .get_direct_descendants_page(
        execution_id,
        options.root.clone(),
        None,
        options.max_default_nodes,
      )
      .await?;

    let collapsed_groups = options.collapsed.clone();
    let get_expandable_descendant_group_ids =
      |descendants: &GroupDirectDescendantsPage| -> Vec<GroupId> {
        descendants
          .descendants
          .iter()
          .filter_map(|node| match node {
            GroupTreeNode::Group(group) => Some(group),
            _ => None,
          })
          .map(|node| node.group_id())
          .filter(|group_id| !collapsed_groups.contains(group_id))
          .cloned()
          .collect()
      };
    let root_key = root_group_id(&options.root);
    let mut all_levels = HashMap::new();
    let mut current_level_ids = get_expandable_descendant_group_ids(&root_children);
    let mut remaining_nodes = options
      .max_default_nodes
      .saturating_sub(root_children.pagination.item_count);
    all_levels.insert(root_key.clone(), root_children);
    while remaining_nodes > 0 && !current_level_ids.is_empty() {
      let Some(next_level) = self
        .get_all_direct_descendants_bounded(
          execution_id,
          &current_level_ids,
          remaining_nodes,
        )
        .await?
      else {
        break;
      };

      let next_level_size = next_level.values().map(|g| g.descendants.len()).sum();
      remaining_nodes = remaining_nodes.saturating_sub(next_level_size);
      current_level_ids = next_level
        .values()
        .flat_map(get_expandable_descendant_group_ids)
        .collect();
      all_levels.extend(next_level);
    }

    for expanded in options.expanded {
      let ancestors = self.get_groups_to_ancestor(&expanded, &options.root)?;
      for ancestor in ancestors {
        let ancestor_id = ancestor.group_ids.first().cloned();
        if let Some(ancestor_id) = ancestor_id {
          let descendants = self
            .get_direct_descendants_page(
              execution_id,
              Some(ancestor_id.clone()),
              None,
              options.page_size,
            )
            .await?;
          all_levels.insert(ancestor_id, descendants);
        }
      }
    }

    Ok(all_levels)
  }
}

impl SledStorage {
  /// Attempts to get all direct descendants of the input nodes but exits early
  /// if the total descendants exceeds the max.
  async fn get_all_direct_descendants_bounded(
    &self,
    execution_id: ExecutionId,
    current_level: &Vec<GroupId>,
    max_nodes: usize,
  ) -> StorageResult<Option<HashMap<GroupId, GroupDirectDescendantsPage>>> {
    let mut results = HashMap::new();
    let mut remaining_nodes = max_nodes;
    for node in current_level {
      let node_descendants = self
        .get_direct_descendants_page(execution_id, Some(node.clone()), None, remaining_nodes)
        .await?;
      if node_descendants.pagination.item_count > remaining_nodes {
        return Ok(None);
      }
      remaining_nodes -= node_descendants.pagination.item_count;
      results.insert(node.clone(), node_descendants);
    }
    Ok(Some(results))
  }

  fn get_groups_to_ancestor(
    &self,
    group_id: &GroupId,
    parent: &Option<GroupId>,
  ) -> StorageResult<Vec<observation_tools_shared::Observation>> {
    let mut current_group_id = Some(group_id.clone());
    let mut ancestors = vec![];
    while let Some(group_id) = current_group_id {
      let current_group = self
        .get_group(&group_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Group {:?} does not exist", group_id)))?;
      if ancestors
        .iter()
        .any(|g: &observation_tools_shared::Observation| g.group_ids.contains(&group_id))
      {
        return Err(StorageError::Internal(format!(
          "Cycle detected {:?} -> {:?}",
          group_id,
          ancestors
            .last()
            .and_then(|g: &observation_tools_shared::Observation| g.group_ids.first())
        )));
      }
      current_group_id = current_group.parent_group_id.clone();
      ancestors.push(current_group);
      if current_group_id == *parent {
        break;
      }
    }
    Ok(ancestors)
  }

  fn get_group(
    &self,
    group_id: &GroupId,
  ) -> StorageResult<Option<observation_tools_shared::Observation>> {
    let gi_tree = self.group_index_tree()?;
    let obs_id_bytes = match gi_tree.get(group_id.as_str().as_bytes())? {
      Some(bytes) => bytes,
      None => return Ok(None),
    };
    let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID encoding: {}", e)))?;
    let obs_id = ObservationId::parse(&obs_id_str)
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;

    let obs_tree = self.observations_tree()?;
    let key = metadata_key(&obs_id);
    let value = match obs_tree.get(key.as_bytes())? {
      Some(v) => v,
      None => return Ok(None),
    };
    let stored = StoredObservation::decode(value.as_ref())?;
    Ok(Some(stored.to_observation()?))
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
  ) -> StorageResult<(Vec<(StoredGroupChild, observation_tools_shared::Observation)>, Option<String>)> {
    let prefix = GroupChildrenKey::encode_prefix(execution_id, parent_id);

    let iter: Box<dyn Iterator<Item = sled::Result<(sled::IVec, sled::IVec)>>> =
      if let Some(token) = page_token {
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
        let stored = StoredObservation::decode(v.as_ref())?;
        let obs = stored.to_observation()?;
        count += 1;
        if count <= limit {
          results.push((child, obs));
        } else {
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
    obs: observation_tools_shared::Observation,
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
        observation: obs,
        group_ancestors: ancestors,
        content,
      }))
    } else {
      Ok(GroupTreeNode::Observation(obs))
    }
  }

  /// Walk up the parent chain to compute group ancestors (oldest ancestor first)
  fn compute_group_ancestors(
    &self,
    obs_tree: &sled::Tree,
    group_obs: &observation_tools_shared::Observation,
  ) -> StorageResult<Vec<GroupId>> {
    let gi_tree = self.group_index_tree()?;
    let mut ancestors = Vec::new();
    let mut current_parent = group_obs.parent_group_id.clone();

    while let Some(ref parent_id) = current_parent {
      if ancestors.iter().any(|a: &GroupId| a.as_str() == parent_id.as_str()) {
        break;
      }
      ancestors.push(parent_id.clone());

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

    ancestors.reverse();
    Ok(ancestors)
  }
}

#[cfg(test)]
mod tests {
  use super::super::test_helpers::*;
  use crate::storage::make_group_tree;
  use crate::storage::root_group_id;
  use crate::storage::GroupMembershipOptions;
  use crate::storage::GroupTree;
  use crate::storage::GroupTreeNode;
  use crate::storage::{ExecutionStorage, GroupStorage, ObservationStorage, PAGE_SIZE};
  use observation_tools_shared::GroupId;
  use std::collections::HashSet;

  #[tokio::test]
  async fn test_get_direct_descendants_root() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let obs1 = make_observation(exec.id, "root-obs-1");
    let obs2 = make_observation(exec.id, "root-obs-2");
    storage
      .store_observations(vec![obs1, obs2])
      .await
      .expect("store obs");

    let page = storage
      .get_direct_descendants_page(exec.id, None, None, PAGE_SIZE)
      .await
      .expect("get descendants");
    assert_eq!(page.descendants.len(), 2);
  }

  #[tokio::test]
  async fn test_get_descendants_simple() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let group = make_group(exec.id, "parent-group", None);
    let group_id = GroupId::from(group.id.to_string());

    let child1 = make_obs_in_group(exec.id, "child-1", group_id.clone());
    let child2 = make_obs_in_group(exec.id, "child-2", group_id.clone());

    storage
      .store_observations(vec![group, child1, child2])
      .await
      .expect("store obs");

    let options = GroupMembershipOptions {
      root: None,
      expanded: HashSet::new(),
      collapsed: HashSet::new(),
      max_default_nodes: 100,
      page_size: 100,
    };
    let root_key = root_group_id(&options.root);
    let data = storage
      .get_descendants(exec.id, options)
      .await
      .expect("get descendants");
    let tree = make_group_tree(data, root_key);

    match tree {
      GroupTree::Tree { roots } => {
        assert_eq!(roots.len(), 1);
        match &roots[0] {
          GroupTreeNode::Group(g) => {
            assert_eq!(g.observation.name, "parent-group");
            assert_eq!(g.content.descendants.len(), 2);
          }
          _ => panic!("expected group node"),
        }
      }
      GroupTree::List(_) => panic!("expected tree, got list"),
    }
  }

  #[tokio::test]
  async fn test_get_descendants_exceeds_max() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let mut obs = Vec::new();
    for i in 0..10 {
      obs.push(make_observation(exec.id, &format!("obs-{}", i)));
    }
    storage.store_observations(obs).await.expect("store obs");

    let options = GroupMembershipOptions {
      root: None,
      expanded: HashSet::new(),
      collapsed: HashSet::new(),
      max_default_nodes: 5,
      page_size: 100,
    };
    let root_key = root_group_id(&options.root);
    let data = storage
      .get_descendants(exec.id, options)
      .await
      .expect("get descendants");
    let tree = make_group_tree(data, root_key);

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

    let grandparent = make_group(exec.id, "grandparent", None);
    let gp_id = GroupId::from(grandparent.id.to_string());

    let parent = make_group(exec.id, "parent", Some(gp_id.clone()));
    let p_id = GroupId::from(parent.id.to_string());

    let child = make_obs_in_group(exec.id, "leaf", p_id.clone());

    storage
      .store_observations(vec![grandparent, parent, child])
      .await
      .expect("store obs");

    let options = GroupMembershipOptions {
      root: None,
      expanded: HashSet::new(),
      collapsed: HashSet::new(),
      max_default_nodes: 100,
      page_size: 100,
    };
    let root_key = root_group_id(&options.root);
    let data = storage
      .get_descendants(exec.id, options)
      .await
      .expect("get descendants");
    let tree = make_group_tree(data, root_key);

    match tree {
      GroupTree::Tree { roots } => {
        assert_eq!(roots.len(), 1);
        match &roots[0] {
          GroupTreeNode::Group(g) => {
            assert_eq!(g.observation.name, "grandparent");
            assert_eq!(g.content.descendants.len(), 1);
            match &g.content.descendants[0] {
              GroupTreeNode::Group(inner) => {
                assert_eq!(inner.observation.name, "parent");
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
}
