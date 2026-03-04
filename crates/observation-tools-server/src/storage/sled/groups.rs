//! Group tree storage methods for SledStorage

use super::keys::GroupChildrenKey;
use super::metadata_key;
use super::SledStorage;
use super::ROOT_SENTINEL;
use crate::storage::proto::StoredGroupChild;
use crate::storage::proto::StoredObservation;
use crate::storage::Group;
use crate::storage::GroupDirectDescendantsPage;
use crate::storage::GroupTree;
use crate::storage::GroupTreeNode;
use crate::storage::ObservationWithPayloads;
use crate::storage::PaginationInfo;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use crate::storage::PAGE_SIZE;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use prost::Message;
use std::collections::VecDeque;

impl SledStorage {
  /// Walk up the parent chain to compute group ancestors (oldest ancestor first)
  pub(crate) fn compute_group_ancestors(
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
  pub(crate) fn scan_direct_descendants(
    &self,
    gc_tree: &sled::Tree,
    obs_tree: &sled::Tree,
    execution_id: &ExecutionId,
    parent_id: &str,
    page_token: Option<&str>,
    limit: usize,
  ) -> StorageResult<(Vec<(StoredGroupChild, ObservationWithPayloads)>, Option<String>)> {
    let prefix = GroupChildrenKey::encode_prefix(execution_id, parent_id);

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
        let obs = self.decode_metadata_only(&child_obs_id, &v)?;
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
  pub(crate) fn build_tree_node(
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

  pub(super) fn get_direct_descendants_page_impl(
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

  pub(super) fn get_group_tree_bfs_impl(
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
    let prefix = GroupChildrenKey::encode_prefix(&execution_id, parent_id);
    let root_children: Vec<(StoredGroupChild, ObservationWithPayloads)> = gc_tree
      .scan_prefix(prefix.as_bytes())
      .filter_map(|item| {
        let (_key, value) = item.ok()?;
        let child = StoredGroupChild::decode(value.as_ref()).ok()?;
        let child_obs_id = ObservationId::parse(&child.child_id).ok()?;
        let key = metadata_key(&child_obs_id);
        let obs_value = obs_tree.get(key.as_bytes()).ok()??;
        let obs = self.decode_metadata_only(&child_obs_id, &obs_value).ok()?;
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
      let child_prefix = GroupChildrenKey::encode_prefix(&execution_id, &group_child_id);
      let children: Vec<(StoredGroupChild, ObservationWithPayloads)> = gc_tree
        .scan_prefix(child_prefix.as_bytes())
        .filter_map(|item| {
          let (_key, value) = item.ok()?;
          let child = StoredGroupChild::decode(value.as_ref()).ok()?;
          let child_obs_id = ObservationId::parse(&child.child_id).ok()?;
          let key = metadata_key(&child_obs_id);
          let obs_value = obs_tree.get(key.as_bytes()).ok()??;
          let obs = self.decode_metadata_only(&child_obs_id, &obs_value).ok()?;
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
  use super::super::test_helpers::*;
  use crate::storage::GroupTree;
  use crate::storage::GroupTreeNode;
  use crate::storage::MetadataStorage;
  use observation_tools_shared::GroupId;

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
    let group_id = GroupId::from(group.id.to_string());

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
    let gp_id = GroupId::from(grandparent.id.to_string());

    let parent = make_group(exec.id, "parent", Some(gp_id.clone()));
    let p_id = GroupId::from(parent.id.to_string());

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
}
