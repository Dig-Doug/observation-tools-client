//! Group tree view models and builder for the payload tab

use crate::api::observations::GetObservation;
use crate::storage::GroupTreeNode;
use crate::storage::MetadataStorage;
use crate::storage::StorageError;
use observation_tools_shared::models::ExecutionId;
use observation_tools_shared::GroupId;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

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
    fn page_token_for(&self, group_id: Option<&str>) -> Option<&str> {
        let key = match group_id {
            Some(id) => format!("page.{}", id),
            None => "page._root_".to_string(),
        };
        self.extra.get(&key).map(|s| s.as_str())
    }

    /// Parse the expanded set from the comma-separated string
    fn expanded_set(&self) -> HashSet<String> {
        self.expanded
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
        self.extra
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
    pub is_expanded: bool,
    pub can_expand: bool,
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

    // Get root-level page token
    let root_page_token = query
        .page_token_for(focus_group_id.as_ref().map(|g| g.as_str()))
        .map(|s| s.to_string());

    // Fetch direct descendants at root (or focused group)
    let root_page = metadata
        .get_direct_descendants_page(execution_id, focus_group_id.clone(), root_page_token)
        .await?;

    let root_pagination = to_pagination_view(
        &root_page.pagination,
        execution_id,
        focus_group_id.as_ref().map(|g| g.as_str()),
        query,
    );

    node_count += root_page.descendants.len();

    // Recursively build children
    let nodes = build_nodes_recursive(
        metadata,
        execution_id,
        root_page.descendants,
        &expanded_set,
        query,
        &url_ctx,
        &mut node_count,
    )
    .await?;

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

/// Recursively build TreeNodeView from storage nodes, expanding groups in the expanded set
fn build_nodes_recursive<'a>(
    metadata: &'a Arc<dyn MetadataStorage>,
    execution_id: ExecutionId,
    nodes: Vec<GroupTreeNode>,
    expanded_set: &'a HashSet<String>,
    query: &'a PayloadTreeQuery,
    url_ctx: &'a UrlContext<'a>,
    node_count: &'a mut usize,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<Vec<TreeNodeView>, crate::api::AppError>>
            + Send
            + 'a,
    >,
> {
    Box::pin(async move {
        let mut result = Vec::new();

        for node in nodes {
            match node {
                GroupTreeNode::Group(group) => {
                    // Use group_ids[0] if available (the group's own GroupId),
                    // otherwise fall back to observation ID for backward compatibility
                    let group_id_str = group
                        .metadata
                        .observation
                        .group_ids
                        .first()
                        .map(|g| g.as_str().to_string())
                        .unwrap_or_else(|| group.metadata.observation.id.to_string());
                    let is_expanded = expanded_set.contains(&group_id_str);

                    let (children, child_pagination) = if is_expanded {
                        // Check budget before expanding
                        if *node_count + TREE_PAGE_SIZE < TREE_NODE_BUDGET {
                            let page_token = query
                                .page_token_for(Some(&group_id_str))
                                .map(|s| s.to_string());

                            let child_page = metadata
                                .get_direct_descendants_page(
                                    execution_id,
                                    Some(GroupId::from(group_id_str.clone())),
                                    page_token,
                                )
                                .await?;

                            *node_count += child_page.descendants.len();

                            let child_pag = to_pagination_view(
                                &child_page.pagination,
                                execution_id,
                                Some(&group_id_str),
                                query,
                            );

                            let children = build_nodes_recursive(
                                metadata,
                                execution_id,
                                child_page.descendants,
                                expanded_set,
                                query,
                                url_ctx,
                                node_count,
                            )
                            .await?;

                            (children, child_pag)
                        } else {
                            // Budget exceeded - can't expand
                            (Vec::new(), None)
                        }
                    } else {
                        (Vec::new(), None)
                    };

                    let can_expand =
                        !is_expanded && *node_count + TREE_PAGE_SIZE < TREE_NODE_BUDGET;

                    let obs_id_str = group.metadata.observation.id.to_string();
                    let expand_url = if can_expand {
                        Some(url_ctx.expand_url(&group_id_str))
                    } else {
                        None
                    };
                    let collapse_url = if is_expanded {
                        Some(url_ctx.collapse_url(&group_id_str))
                    } else {
                        None
                    };
                    let focus_url = Some(url_ctx.focus_url(&group_id_str));
                    let select_url = url_ctx.select_url(&obs_id_str);

                    result.push(TreeNodeView {
                        node_type: "group".to_string(),
                        observation: GetObservation::new(group.metadata),
                        group_id: Some(group_id_str),
                        is_expanded,
                        can_expand,
                        children,
                        pagination: child_pagination,
                        expand_url,
                        collapse_url,
                        focus_url,
                        select_url,
                    });
                }
                GroupTreeNode::Observation(obs) => {
                    let obs_id_str = obs.observation.id.to_string();
                    let select_url = url_ctx.select_url(&obs_id_str);

                    result.push(TreeNodeView {
                        node_type: "observation".to_string(),
                        observation: GetObservation::new(obs),
                        group_id: None,
                        is_expanded: false,
                        can_expand: false,
                        children: Vec::new(),
                        pagination: None,
                        expand_url: None,
                        collapse_url: None,
                        focus_url: None,
                        select_url,
                    });
                }
            }
        }

        Ok(result)
    })
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
    let mut current_parent = obs.observation.parent_group_id.clone();

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
                    parent_obs.observation.name.clone(),
                ));
                current_parent = parent_obs.observation.parent_group_id.clone();
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
    group_id: Option<&str>,
    query: &PayloadTreeQuery,
) -> Option<PaginationView> {
    if pagination.previous_page_token.is_none() && pagination.next_page_token.is_none() {
        return None;
    }

    let page_key = match group_id {
        Some(id) => format!("page.{}", id),
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
