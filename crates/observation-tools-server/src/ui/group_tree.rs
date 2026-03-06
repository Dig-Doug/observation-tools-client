//! Group tree view models and builder for the payload tab

use crate::api::observations::GetObservation;
use crate::storage::MetadataStorage;
use crate::storage::GroupMembershipOptions;
use crate::storage::StorageError;
use crate::storage::{make_group_tree, root_group_id, GroupTree, GroupTreeNode};
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::GroupId;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

/// Maximum number of tree nodes to render before refusing further expansion
const TREE_NODE_BUDGET: usize = 500;

/// Page size for tree node pagination
const TREE_PAGE_SIZE: usize = 100;

/// Query parameters for the payload tree view
#[derive(Debug, serde::Deserialize)]
pub struct PayloadTreeQuery {
  /// Focus on a specific group subtree
  pub focus: Option<String>,
  /// Comma-separated list of expanded group IDs
  pub expanded: Option<String>,
  /// Selected observation for side panel
  pub obs: Option<String>,
  /// Captures page.{id}=token entries
  #[serde(flatten)]
  pub extra: HashMap<String, String>,
}

impl PayloadTreeQuery {
  /// Get the page token for a given group ID (or root)
  fn page_token_for(&self, group_id: &Option<GroupId>) -> Option<&str> {
    let key = match group_id {
      Some(id) => format!("page.{}", id.as_str()),
      None => "page._root_".to_string(),
    };
    self.extra.get(&key).map(|s| s.as_str())
  }

  /// Parse the expanded set from the comma-separated string
  fn expanded_set(&self) -> HashSet<String> {
    self
      .expanded
      .as_deref()
      .unwrap_or("")
      .split(',')
      .filter(|s| !s.is_empty())
      .map(|s| s.to_string())
      .collect()
  }

  /// Return expanded string with a group ID added
  fn expanded_with(&self, group_id: &str) -> String {
    let mut set = self.expanded_set();
    set.insert(group_id.to_string());
    let mut ids: Vec<_> = set.into_iter().collect();
    ids.sort();
    ids.join(",")
  }

  /// Return expanded string with a group ID removed
  fn expanded_without(&self, group_id: &str) -> String {
    let mut set = self.expanded_set();
    set.remove(group_id);
    let mut ids: Vec<_> = set.into_iter().collect();
    ids.sort();
    ids.join(",")
  }

  /// Collect page tokens from extra params
  fn page_tokens(&self) -> HashMap<String, String> {
    self
      .extra
      .iter()
      .filter(|(k, _)| k.starts_with("page."))
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect()
  }
}

// -- View models --

#[derive(Debug, Serialize)]
pub struct GroupTreeView {
  pub nodes: Vec<TreeNodeView>,
  pub pagination: Option<PaginationView>,
  pub breadcrumbs: Vec<BreadcrumbView>,
  pub focused_group_id: Option<String>,
  pub node_count: usize,
  pub node_budget: usize,
  /// URL to close side panel (current state without obs)
  pub close_url: String,
}

#[derive(Debug, Serialize)]
pub struct TreeNodeView {
  pub node_type: String,
  pub observation: GetObservation,
  pub group_id: Option<String>,
  pub can_expand: bool,
  /// Only set if the group is expanded
  pub children: Vec<TreeNodeView>,
  pub pagination: Option<PaginationView>,
  /// URL to expand this group (adds to expanded set)
  pub expand_url: Option<String>,
  /// URL to collapse this group (removes from expanded set)
  pub collapse_url: Option<String>,
  /// URL to focus on this group
  pub focus_url: Option<String>,
  /// URL to select this observation in side panel
  pub select_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationView {
  pub item_count: usize,
  pub prev: Option<String>,
  pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreadcrumbView {
  pub group_id: String,
  pub name: String,
  /// URL to focus on this ancestor
  pub url: String,
}

// -- URL builder --

/// Build a payload tree URL from parameters.
/// All values (IDs, page tokens) are UUID hex strings, so no percent-encoding needed.
fn build_tree_url(
  execution_id: &str,
  focus: Option<&str>,
  expanded: Option<&str>,
  obs: Option<&str>,
  page_tokens: &HashMap<String, String>,
) -> String {
  let mut params = Vec::new();

  if let Some(f) = focus {
    if !f.is_empty() {
      params.push(format!("focus={}", f));
    }
  }
  if let Some(e) = expanded {
    if !e.is_empty() {
      params.push(format!("expanded={}", e));
    }
  }
  if let Some(o) = obs {
    if !o.is_empty() {
      params.push(format!("obs={}", o));
    }
  }
  for (key, value) in page_tokens {
    params.push(format!("{}={}", key, value));
  }

  let base = format!("/exe/{}/payload", execution_id);
  if params.is_empty() {
    base
  } else {
    format!("{}?{}", base, params.join("&"))
  }
}

/// Context for building URLs during tree construction
struct UrlContext<'a> {
  exec_id_str: String,
  query: &'a PayloadTreeQuery,
  page_tokens: HashMap<String, String>,
}

impl<'a> UrlContext<'a> {
  fn new(execution_id: ExecutionId, query: &'a PayloadTreeQuery) -> Self {
    Self {
      exec_id_str: execution_id.to_string(),
      query,
      page_tokens: query.page_tokens(),
    }
  }

  /// URL with a group added to expanded
  fn expand_url(&self, group_id: &str) -> String {
    let expanded = self.query.expanded_with(group_id);
    build_tree_url(
      &self.exec_id_str,
      self.query.focus.as_deref(),
      Some(&expanded),
      self.query.obs.as_deref(),
      &self.page_tokens,
    )
  }

  /// URL with a group removed from expanded
  fn collapse_url(&self, group_id: &str) -> String {
    let expanded = self.query.expanded_without(group_id);
    build_tree_url(
      &self.exec_id_str,
      self.query.focus.as_deref(),
      if expanded.is_empty() {
        None
      } else {
        Some(&expanded)
      },
      self.query.obs.as_deref(),
      &self.page_tokens,
    )
  }

  /// URL to focus on a group (clears expanded and page tokens)
  fn focus_url(&self, group_id: &str) -> String {
    build_tree_url(
      &self.exec_id_str,
      Some(group_id),
      None,
      self.query.obs.as_deref(),
      &HashMap::new(),
    )
  }

  /// URL to select an observation in the side panel
  fn select_url(&self, obs_id: &str) -> String {
    build_tree_url(
      &self.exec_id_str,
      self.query.focus.as_deref(),
      self.query.expanded.as_deref(),
      Some(obs_id),
      &self.page_tokens,
    )
  }

  /// URL for the root breadcrumb
  fn root_url(&self) -> String {
    build_tree_url(
      &self.exec_id_str,
      None,
      None,
      self.query.obs.as_deref(),
      &HashMap::new(),
    )
  }

  /// URL without obs param (for close button)
  fn close_url(&self) -> String {
    build_tree_url(
      &self.exec_id_str,
      self.query.focus.as_deref(),
      self.query.expanded.as_deref(),
      None,
      &self.page_tokens,
    )
  }
}

// -- Tree builder --

/// Build the group tree view model for the payload tab
pub async fn build_group_tree_view(
  metadata: &Arc<dyn MetadataStorage>,
  execution_id: ExecutionId,
  query: &PayloadTreeQuery,
) -> Result<GroupTreeView, crate::api::AppError> {
  let expanded_set = query.expanded_set();
  let url_ctx = UrlContext::new(execution_id, query);
  let mut node_count: usize = 0;

  // Determine the focus group
  let focus_group_id = query
    .focus
    .as_deref()
    .filter(|s| !s.is_empty())
    .map(GroupId::from);

  // Build breadcrumbs if focused
  let breadcrumbs = if let Some(ref focus_id) = focus_group_id {
    build_breadcrumbs(metadata, focus_id, &url_ctx).await?
  } else {
    Vec::new()
  };

  let root_page = if let Some(root_page_token) = query.page_token_for(&focus_group_id) {
    let page = metadata
      .get_direct_descendants_page(
        execution_id,
        focus_group_id.clone(),
        Some(root_page_token.to_string()),
        TREE_PAGE_SIZE,
      )
      .await?;
    GroupTree::List(page)
  } else {
    let options = GroupMembershipOptions {
      root: focus_group_id.clone(),
      expanded: HashSet::new(),
      collapsed: HashSet::new(),
      max_default_nodes: TREE_NODE_BUDGET,
      page_size: TREE_PAGE_SIZE,
    };
    let root_key = root_group_id(&options.root);
    let data = metadata.get_descendants(execution_id, options).await?;
    make_group_tree(data, root_key)
  };
  info!("{:#?}", root_page);

  let (nodes, pagination) = match root_page {
    GroupTree::List(nodes) => (nodes.descendants, Some(nodes.pagination)),
    GroupTree::Tree { roots } => (roots, None),
  };
  node_count += nodes.len();

  let nodes = nodes
    .into_iter()
    .map(|n| build_node_recursive2(execution_id, &url_ctx, query, n))
    .collect();
  let root_pagination =
    pagination.and_then(|p| to_pagination_view(&p, execution_id, &focus_group_id, query));

  // Recursively build children
  Ok(GroupTreeView {
    nodes,
    pagination: root_pagination,
    breadcrumbs,
    focused_group_id: focus_group_id.map(|g| g.as_str().to_string()),
    node_count,
    node_budget: TREE_NODE_BUDGET,
    close_url: url_ctx.close_url(),
  })
}

fn build_node_recursive2<'a>(
  execution_id: ExecutionId,
  url_ctx: &'a UrlContext<'a>,
  query: &'a PayloadTreeQuery,
  node: GroupTreeNode,
) -> TreeNodeView {
  match node {
    GroupTreeNode::Group(group) => {
      let group_id = group.observation.group_ids.first().cloned();
      let child_pag = to_pagination_view(&group.content.pagination, execution_id, &group_id, query);
      let obs_id_str = group.observation.id.to_string();
      let select_url = url_ctx.select_url(&obs_id_str);
      TreeNodeView {
        node_type: "group".to_string(),
        observation: GetObservation::new(group.observation, Vec::new()),
        group_id: group_id.map(|g| g.as_str().to_string()),
        can_expand: false,
        children: group
          .content
          .descendants
          .into_iter()
          .map(|d| build_node_recursive2(execution_id, url_ctx, query, d))
          .collect(),
        pagination: child_pag,
        expand_url: None,
        collapse_url: None,
        focus_url: None,
        select_url,
      }
    }
    GroupTreeNode::Observation(obs) => {
      let obs_id_str = obs.id.to_string();
      let select_url = url_ctx.select_url(&obs_id_str);
      TreeNodeView {
        node_type: "observation".to_string(),
        observation: GetObservation::new(obs, Vec::new()),
        group_id: None,
        can_expand: false,
        children: Vec::new(),
        pagination: None,
        expand_url: None,
        collapse_url: None,
        focus_url: None,
        select_url,
      }
    }
  }
}

/// Build breadcrumbs for a focused group by walking up the ancestor chain
async fn build_breadcrumbs(
  metadata: &Arc<dyn MetadataStorage>,
  focus_id: &GroupId,
  url_ctx: &UrlContext<'_>,
) -> Result<Vec<BreadcrumbView>, crate::api::AppError> {
  // Look up the group observation via the group_id index
  let obs = match metadata.get_observation_by_group_id(focus_id.clone()).await {
    Ok(obs) => obs,
    Err(StorageError::NotFound(_)) => return Ok(Vec::new()),
    Err(e) => return Err(e.into()),
  };

  // Always include root breadcrumb
  let mut crumbs = vec![BreadcrumbView {
    group_id: String::new(),
    name: "root".to_string(),
    url: url_ctx.root_url(),
  }];

  let mut ancestors = Vec::new();
  let mut current_parent = obs.parent_group_id.clone();

  while let Some(ref parent_id) = current_parent {
    // Prevent infinite loops
    if ancestors
      .iter()
      .any(|(id, _): &(String, String)| id == parent_id.as_str())
    {
      break;
    }

    // Look up the parent group observation via the group_id index
    match metadata
      .get_observation_by_group_id(parent_id.clone())
      .await
    {
      Ok(parent_obs) => {
        ancestors.push((
          parent_id.as_str().to_string(),
          parent_obs.name.clone(),
        ));
        current_parent = parent_obs.parent_group_id.clone();
      }
      Err(_) => break,
    }
  }

  ancestors.reverse(); // oldest ancestor first
  for (group_id, name) in ancestors {
    let url = url_ctx.focus_url(&group_id);
    crumbs.push(BreadcrumbView {
      group_id,
      name,
      url,
    });
  }

  Ok(crumbs)
}

/// Convert storage PaginationInfo to a PaginationView with proper URLs
fn to_pagination_view(
  pagination: &crate::storage::PaginationInfo,
  execution_id: ExecutionId,
  group_id: &Option<GroupId>,
  query: &PayloadTreeQuery,
) -> Option<PaginationView> {
  if pagination.previous_page_token.is_none() && pagination.next_page_token.is_none() {
    return None;
  }

  let page_key = match group_id {
    Some(id) => format!("page.{}", id.as_str()),
    None => "page._root_".to_string(),
  };

  let exec_id_str = execution_id.to_string();

  let build_url = |token: &str| -> String {
    let mut page_tokens: HashMap<String, String> = query
      .extra
      .iter()
      .filter(|(k, _)| k.starts_with("page.") && *k != &page_key)
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    page_tokens.insert(page_key.clone(), token.to_string());

    build_tree_url(
      &exec_id_str,
      query.focus.as_deref(),
      query.expanded.as_deref(),
      query.obs.as_deref(),
      &page_tokens,
    )
  };

  Some(PaginationView {
    item_count: pagination.item_count,
    prev: pagination.previous_page_token.as_deref().map(build_url),
    next: pagination.next_page_token.as_deref().map(build_url),
  })
}
