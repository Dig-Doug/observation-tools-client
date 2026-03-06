use super::keys::GroupChildrenKey;
use super::SledStorage;
use super::ROOT_SENTINEL;
use crate::storage::proto::StoredGroupChild;
use crate::storage::root_group_id;
use crate::storage::GroupDirectDescendantsPage;
use crate::storage::GroupMembershipOptions;
use crate::storage::GroupStorage;
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
    let prefix = GroupChildrenKey::encode_prefix(&execution_id, parent_id);

    let start_key = match page_token {
      Some(ref token) => format!("{}{}\x00", prefix, token),
      None => prefix.clone(),
    };

    let mut descendants = Vec::new();

    for item in gc_tree
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
      let (_key, value) = item?;
      let child = StoredGroupChild::decode(value.as_ref())?;

      let child_obs_id = ObservationId::parse(&child.child_id)
        .map_err(|e| StorageError::Internal(format!("Invalid child ID: {}", e)))?;
      if let Some(obs) = self.decode_observation(&obs_tree, &child_obs_id)? {
        descendants.push(obs);
        if descendants.len() > page_size {
          break;
        }
      }
    }

    let next_page_token = if descendants.len() > page_size {
      descendants.pop();
      descendants.last().map(|o| o.id.to_string())
    } else {
      None
    };

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
    let gi_tree = self.group_index_tree()?;
    let obs_id_bytes = gi_tree
      .get(group_id.as_str().as_bytes())?
      .ok_or_else(|| StorageError::NotFound(format!("Group {} not found", group_id.as_str())))?;
    let obs_id_str = String::from_utf8(obs_id_bytes.to_vec())
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID encoding: {}", e)))?;
    let obs_id = ObservationId::parse(&obs_id_str)
      .map_err(|e| StorageError::Internal(format!("Invalid observation ID: {}", e)))?;

    let obs_tree = self.observations_tree()?;
    self
      .decode_observation(&obs_tree, &obs_id)?
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
          .filter(|obs| obs.observation_type == observation_tools_shared::ObservationType::Group)
          .filter_map(|obs| obs.group_ids.first())
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
      let ancestors = self.get_groups_to_ancestor(&expanded, &options.root).await?;
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

  async fn get_groups_to_ancestor(
    &self,
    group_id: &GroupId,
    parent: &Option<GroupId>,
  ) -> StorageResult<Vec<observation_tools_shared::Observation>> {
    let mut current_group_id = Some(group_id.clone());
    let mut ancestors = vec![];
    while let Some(group_id) = current_group_id {
      let current_group = self.get_observation_by_group_id(group_id.clone()).await?;
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
}

#[cfg(test)]
mod tests {
  use super::super::test_helpers::*;
  use crate::storage::root_group_id;
  use crate::storage::GroupMembershipOptions;
  use crate::storage::{ExecutionStorage, GroupStorage, ObservationStorage, PAGE_SIZE};
  use observation_tools_shared::GroupId;
  use observation_tools_shared::ObservationType;
  use std::collections::HashSet;

  #[tokio::test]
  async fn test_get_direct_descendants_empty() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let page = storage
        .get_direct_descendants_page(exec.id, None, None, PAGE_SIZE)
        .await?;
    assert_eq!(page.descendants.len(), 0);
    assert!(page.pagination.next_page_token.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_direct_descendants_root() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs1 = make_observation(exec.id, "root-obs-1");
    let obs2 = make_observation(exec.id, "root-obs-2");
    storage.store_observations(vec![obs1, obs2]).await?;

    let page = storage
        .get_direct_descendants_page(exec.id, None, None, PAGE_SIZE)
        .await?;
    assert_eq!(page.descendants.len(), 2);
    assert!(page.pagination.next_page_token.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_direct_descendants_of_group() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let group = make_group(exec.id, "my-group", None);
    let group_id = GroupId::from("my-group");
    let child1 = make_obs_in_group(exec.id, "child-1", group_id.clone());
    let child2 = make_obs_in_group(exec.id, "child-2", group_id.clone());
    let unrelated = make_observation(exec.id, "unrelated");

    storage
        .store_observations(vec![group, child1, child2, unrelated])
        .await?;

    let page = storage
        .get_direct_descendants_page(exec.id, Some(group_id), None, PAGE_SIZE)
        .await?;
    assert_eq!(page.descendants.len(), 2);
    let names: Vec<_> = page.descendants.iter().map(|o| o.name.as_str()).collect();
    assert!(names.contains(&"child-1"));
    assert!(names.contains(&"child-2"));
    Ok(())
  }

  #[tokio::test]
  async fn test_get_direct_descendants_pagination() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs: Vec<_> = (0..10)
        .map(|i| make_observation(exec.id, &format!("obs-{}", i)))
        .collect();
    storage.store_observations(obs).await?;

    // Page size of 3
    let page1 = storage
        .get_direct_descendants_page(exec.id, None, None, 3)
        .await?;
    assert_eq!(page1.descendants.len(), 3);
    assert!(page1.pagination.next_page_token.is_some());

    let page2 = storage
        .get_direct_descendants_page(exec.id, None, page1.pagination.next_page_token, 3)
        .await?;
    assert_eq!(page2.descendants.len(), 3);
    assert!(page2.pagination.next_page_token.is_some());

    // No overlap
    let page1_ids: std::collections::HashSet<_> =
        page1.descendants.iter().map(|o| o.id).collect();
    for o in &page2.descendants {
      assert!(!page1_ids.contains(&o.id));
    }
    Ok(())
  }

  #[tokio::test]
  async fn test_get_direct_descendants_isolates_executions() -> anyhow::Result<()> {
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

    let page1 = storage
        .get_direct_descendants_page(exec1.id, None, None, PAGE_SIZE)
        .await?;
    assert_eq!(page1.descendants.len(), 1);
    assert_eq!(page1.descendants[0].name, "exec1-obs");

    let page2 = storage
        .get_direct_descendants_page(exec2.id, None, None, PAGE_SIZE)
        .await?;
    assert_eq!(page2.descendants.len(), 1);
    assert_eq!(page2.descendants[0].name, "exec2-obs");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observation_by_group_id() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let group = make_group(exec.id, "my-group", None);
    storage.store_observations(vec![group]).await?;

    let result = storage
        .get_observation_by_group_id(GroupId::from("my-group"))
        .await?;
    assert_eq!(result.name, "my-group");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observation_by_group_id_not_found() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();

    let result = storage
        .get_observation_by_group_id(GroupId::from("nonexistent"))
        .await;
    assert!(result.is_err());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_observation_by_group_id_nested() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let parent = make_group(exec.id, "parent", None);
    let child = make_group(exec.id, "child", Some(GroupId::from("parent")));
    storage.store_observations(vec![parent, child]).await?;

    let result = storage
        .get_observation_by_group_id(GroupId::from("child"))
        .await?;
    assert_eq!(result.name, "child");
    assert_eq!(result.parent_group_id, Some(GroupId::from("parent")));
    Ok(())
  }

  #[tokio::test]
  async fn test_get_groups_to_ancestor_single() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let group = make_group(exec.id, "my-group", None);
    let group_id = GroupId::from("my-group");
    storage.store_observations(vec![group]).await?;

    let result = storage.get_groups_to_ancestor(&group_id, &None).await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "my-group");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_groups_to_ancestor_chain() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let grandparent = make_group(exec.id, "grandparent", None);
    let gp_id = GroupId::from("grandparent");

    let parent = make_group(exec.id, "parent", Some(gp_id.clone()));
    let p_id = GroupId::from("parent");

    let child = make_group(exec.id, "child", Some(p_id.clone()));
    let c_id = GroupId::from("child");

    storage
      .store_observations(vec![grandparent, parent, child])
      .await?;

    // From child, walk to root
    let result = storage.get_groups_to_ancestor(&c_id, &None).await?;
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "child");
    assert_eq!(result[1].name, "parent");
    assert_eq!(result[2].name, "grandparent");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_groups_to_ancestor_stops_at_parent() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let grandparent = make_group(exec.id, "grandparent", None);
    let gp_id = GroupId::from("grandparent");

    let parent = make_group(exec.id, "parent", Some(gp_id.clone()));
    let p_id = GroupId::from("parent");

    let child = make_group(exec.id, "child", Some(p_id.clone()));
    let c_id = GroupId::from("child");

    storage
      .store_observations(vec![grandparent, parent, child])
      .await?;

    // From child, stop at grandparent
    let result = storage.get_groups_to_ancestor(&c_id, &Some(gp_id)).await?;
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "child");
    assert_eq!(result[1].name, "parent");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_groups_to_ancestor_not_found() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();

    let result = storage.get_groups_to_ancestor(&GroupId::from("nonexistent"), &None).await;
    assert!(result.is_err());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_groups_to_ancestor_cycle() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    // Create two groups that reference each other as parents
    let group_a = make_group(exec.id, "group-a", Some(GroupId::from("group-b")));
    let group_b = make_group(exec.id, "group-b", Some(GroupId::from("group-a")));

    storage.store_observations(vec![group_a, group_b]).await?;

    let result = storage.get_groups_to_ancestor(&GroupId::from("group-a"), &None).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Cycle detected"), "expected cycle error, got: {}", err);
    Ok(())
  }

  fn default_options() -> GroupMembershipOptions {
    GroupMembershipOptions {
      root: None,
      expanded: HashSet::new(),
      collapsed: HashSet::new(),
      max_default_nodes: 100,
      page_size: 100,
    }
  }

  #[tokio::test]
  async fn test_get_descendants_empty() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let data = storage.get_descendants(exec.id, default_options()).await?;
    let root_key = root_group_id(&None);
    assert_eq!(data.len(), 1);
    assert!(data[&root_key].descendants.is_empty());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_flat() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs: Vec<_> = (0..3)
      .map(|i| make_observation(exec.id, &format!("obs-{}", i)))
      .collect();
    storage.store_observations(obs).await?;

    let data = storage.get_descendants(exec.id, default_options()).await?;
    let root_key = root_group_id(&None);
    // Only the root entry, no group children to expand
    assert_eq!(data.len(), 1);
    let root_page = &data[&root_key];
    assert_eq!(root_page.descendants.len(), 3);
    for obs in &root_page.descendants {
      assert_ne!(obs.observation_type, ObservationType::Group);
    }
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_nested_groups() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let parent = make_group(exec.id, "parent", None);
    let child = make_obs_in_group(exec.id, "child", GroupId::from("parent"));
    storage.store_observations(vec![parent, child]).await?;

    let data = storage.get_descendants(exec.id, default_options()).await?;
    let root_key = root_group_id(&None);
    let parent_key = GroupId::from("parent");
    // Root has the parent group, parent group has the child
    assert_eq!(data.len(), 2);
    assert_eq!(data[&root_key].descendants.len(), 1);
    assert_eq!(data[&root_key].descendants[0].name, "parent");
    assert_eq!(data[&parent_key].descendants.len(), 1);
    assert_eq!(data[&parent_key].descendants[0].name, "child");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_exceeds_max_nodes() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let obs: Vec<_> = (0..10)
      .map(|i| make_observation(exec.id, &format!("obs-{}", i)))
      .collect();
    storage.store_observations(obs).await?;

    let options = GroupMembershipOptions {
      max_default_nodes: 5,
      ..default_options()
    };
    let data = storage.get_descendants(exec.id, options).await?;
    let root_key = root_group_id(&None);
    // Root page should be truncated with pagination
    assert_eq!(data.len(), 1);
    let root_page = &data[&root_key];
    assert_eq!(root_page.descendants.len(), 5);
    assert!(root_page.pagination.next_page_token.is_some());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_deeply_nested() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let grandparent = make_group(exec.id, "grandparent", None);
    let parent = make_group(exec.id, "parent", Some(GroupId::from("grandparent")));
    let leaf = make_obs_in_group(exec.id, "leaf", GroupId::from("parent"));
    storage
      .store_observations(vec![grandparent, parent, leaf])
      .await?;

    let data = storage.get_descendants(exec.id, default_options()).await?;
    let root_key = root_group_id(&None);
    let gp_key = GroupId::from("grandparent");
    let p_key = GroupId::from("parent");
    // Three levels: root -> grandparent -> parent -> leaf
    assert_eq!(data.len(), 3);
    assert_eq!(data[&root_key].descendants.len(), 1);
    assert_eq!(data[&root_key].descendants[0].name, "grandparent");
    assert_eq!(data[&gp_key].descendants.len(), 1);
    assert_eq!(data[&gp_key].descendants[0].name, "parent");
    assert_eq!(data[&p_key].descendants.len(), 1);
    assert_eq!(data[&p_key].descendants[0].name, "leaf");
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_with_root() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let group = make_group(exec.id, "my-group", None);
    let child1 = make_obs_in_group(exec.id, "child-1", GroupId::from("my-group"));
    let child2 = make_obs_in_group(exec.id, "child-2", GroupId::from("my-group"));
    let unrelated = make_observation(exec.id, "unrelated");
    storage
      .store_observations(vec![group, child1, child2, unrelated])
      .await?;

    let options = GroupMembershipOptions {
      root: Some(GroupId::from("my-group")),
      ..default_options()
    };
    let data = storage.get_descendants(exec.id, options).await?;
    let root_key = GroupId::from("my-group");
    // Only the focused group's children, not the unrelated obs
    assert_eq!(data.len(), 1);
    let page = &data[&root_key];
    assert_eq!(page.descendants.len(), 2);
    let names: Vec<_> = page.descendants.iter().map(|o| o.name.as_str()).collect();
    assert!(names.contains(&"child-1"));
    assert!(names.contains(&"child-2"));
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_collapsed() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    let parent = make_group(exec.id, "parent", None);
    let child = make_obs_in_group(exec.id, "child", GroupId::from("parent"));
    storage.store_observations(vec![parent, child]).await?;

    let options = GroupMembershipOptions {
      collapsed: HashSet::from([GroupId::from("parent")]),
      ..default_options()
    };
    let data = storage.get_descendants(exec.id, options).await?;
    let root_key = root_group_id(&None);
    // Root has the parent group, but parent's children are NOT fetched
    assert_eq!(data.len(), 1);
    assert_eq!(data[&root_key].descendants.len(), 1);
    assert_eq!(data[&root_key].descendants[0].name, "parent");
    assert!(!data.contains_key(&GroupId::from("parent")));
    Ok(())
  }

  #[tokio::test]
  async fn test_get_descendants_expanded() -> anyhow::Result<()> {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await?;

    // Create a deep tree: grandparent -> parent -> leaf
    let grandparent = make_group(exec.id, "grandparent", None);
    let parent = make_group(exec.id, "parent", Some(GroupId::from("grandparent")));
    let leaf = make_obs_in_group(exec.id, "leaf", GroupId::from("parent"));
    storage
      .store_observations(vec![grandparent, parent, leaf])
      .await?;

    // Use a tiny budget so BFS only gets the root level
    let options = GroupMembershipOptions {
      max_default_nodes: 1,
      expanded: HashSet::from([GroupId::from("parent")]),
      ..default_options()
    };
    let data = storage.get_descendants(exec.id, options).await?;
    let root_key = root_group_id(&None);
    // Root has grandparent (from BFS, truncated)
    assert!(data.contains_key(&root_key));
    // Expanded "parent" triggers ancestor walk: parent + grandparent both get fetched
    assert!(data.contains_key(&GroupId::from("grandparent")));
    assert!(data.contains_key(&GroupId::from("parent")));
    let parent_page = &data[&GroupId::from("parent")];
    assert_eq!(parent_page.descendants.len(), 1);
    assert_eq!(parent_page.descendants[0].name, "leaf");
    Ok(())
  }
}
