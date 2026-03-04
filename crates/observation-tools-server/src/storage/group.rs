use super::sled::metadata_key;
use crate::storage::GroupDirectDescendantsPage;
use crate::storage::GroupTreeNode;
use crate::storage::ObservationWithPayloads;
use crate::storage::PaginationInfo;
use crate::storage::SledStorage;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use std::collections::HashMap;
use std::collections::HashSet;

const ROOT_SENTINEL: &str = "_ROOT_";

pub struct GroupMembershipOptions {
  /// The root group to start the BFS from.
  pub root: Option<GroupId>,
  /// The additional groups that should be expanded in the tree.
  pub expanded: HashSet<GroupId>,
  /// The groups that should be collapsed in the returned tree.
  pub collapsed: HashSet<GroupId>,
  /// The max number of nodes to return in the BFS.
  pub max_default_nodes: usize,
  /// The page size of nodes to return in expanded groups.
  pub page_size: usize,
}

impl SledStorage {
  /// Executes a breadth-first search for group descendants and aims to return N
  /// nodes.
  pub fn get_descendants(
    &self,
    execution_id: ExecutionId,
    options: GroupMembershipOptions,
  ) -> StorageResult<HashMap<GroupId, GroupDirectDescendantsPage>> {
    let root_children = self.get_direct_descendants(
      &execution_id,
      options.root.as_ref(),
      None,
      options.max_default_nodes,
    )?;

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
    let mut all_levels = HashMap::new();
    let mut current_level_ids = get_expandable_descendant_group_ids(&root_children);
    let mut remaining_nodes = options
      .max_default_nodes
      .saturating_sub(root_children.pagination.item_count);
    while remaining_nodes > 0 {
      let Some(next_level) = self.get_all_direct_descendants_bounded(
        &execution_id,
        &current_level_ids,
        remaining_nodes,
      )?
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
        let ancestor_id = ancestor.observation.group_ids.first().cloned();
        if let Some(ancestor_id) = ancestor_id {
          let descendants = self.get_direct_descendants(
            &execution_id,
            Some(&ancestor_id),
            None,
            options.page_size,
          )?;
          all_levels.insert(ancestor_id, descendants);
        }
      }
    }

    Ok(all_levels)
  }

  fn get_direct_descendants(
    &self,
    execution_id: &ExecutionId,
    group_id: Option<&GroupId>,
    page_token: Option<&str>,
    page_size: usize,
  ) -> StorageResult<GroupDirectDescendantsPage> {
    let gc_tree = self.group_children_tree()?;
    let obs_tree = self.observations_tree()?;
    let parent_id = group_id.map_or(ROOT_SENTINEL, |g| g.as_str());

    let (results, next_page_token) = self.scan_direct_descendants(
      &gc_tree,
      &obs_tree,
      execution_id,
      parent_id,
      page_token,
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
        previous_page_token: page_token.map(String::from),
        next_page_token,
      },
    })
  }

  /// Attempts to get all direct descendants of the input nodes but exits early
  /// if the total descendants exceeds the max.
  fn get_all_direct_descendants_bounded(
    &self,
    execution_id: &ExecutionId,
    current_level: &Vec<GroupId>,
    max_nodes: usize,
  ) -> StorageResult<Option<HashMap<GroupId, GroupDirectDescendantsPage>>> {
    let mut results = HashMap::new();
    let mut remaining_nodes = max_nodes;
    for node in current_level {
      let Some(node_descendants) =
        self.get_direct_descendants_bounded(execution_id, Some(node), remaining_nodes)?
      else {
        return Ok(None);
      };
      remaining_nodes = remaining_nodes - node_descendants.pagination.item_count;
      results.insert(node.clone(), node_descendants);
    }
    Ok(Some(results))
  }

  /// Helper method to get direct descendants for use cases that require
  /// exit-early capability. This method could potentially be optimized if we
  /// had an O(1) group-size function.
  fn get_direct_descendants_bounded(
    &self,
    execution_id: &ExecutionId,
    group_id: Option<&GroupId>,
    max: usize,
  ) -> StorageResult<Option<GroupDirectDescendantsPage>> {
    let node_descendants = self.get_direct_descendants(execution_id, group_id, None, max)?;
    if node_descendants.pagination.item_count > max {
      return Ok(None);
    }
    Ok(Some(node_descendants))
  }

  fn get_groups_to_ancestor(
    &self,
    group_id: &GroupId,
    parent: &Option<GroupId>,
  ) -> StorageResult<Vec<ObservationWithPayloads>> {
    let mut current_group_id = Some(group_id.clone());
    let mut ancestors = vec![];
    while let Some(group_id) = current_group_id {
      let current_group = self
        .get_group(&group_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Group {:?} does not exist", group_id)))?;
      if ancestors
        .iter()
        .any(|g: &ObservationWithPayloads| g.observation.group_ids.contains(&group_id))
      {
        return Err(StorageError::Internal(format!(
          "Cycle detected {:?} -> {:?}",
          group_id,
          ancestors
            .last()
            .and_then(|g: &ObservationWithPayloads| g.observation.group_ids.first())
        )));
      }
      current_group_id = current_group.observation.parent_group_id.clone();
      ancestors.push(current_group);
      if current_group_id == *parent {
        break;
      }
    }
    Ok(ancestors)
  }

  fn get_group(&self, group_id: &GroupId) -> StorageResult<Option<ObservationWithPayloads>> {
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
    Ok(Some(self.decode_metadata_only(&obs_id, &value)?))
  }
}
