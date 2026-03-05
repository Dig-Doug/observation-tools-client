//! Storage layer abstractions and implementations

pub mod blob;
pub mod proto;
pub mod sled;
pub mod stored_execution;

pub use blob::BlobStorage;
pub use blob::LocalBlobStorage;
pub use self::sled::SledStorage;
use observation_tools_shared::Execution;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::GroupId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::PayloadId;
use std::collections::HashMap;
use std::collections::HashSet;
use thiserror::Error;

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
  #[error("Not found: {0}")]
  NotFound(String),

  #[error("Already exists: {0}")]
  AlreadyExists(String),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("Database error: {0}")]
  Database(#[from] ::sled::Error),

  #[error("Protobuf decode error: {0}")]
  Protobuf(#[from] prost::DecodeError),

  #[error("Search error: {0}")]
  Search(String),

  #[error("Internal error: {0}")]
  Internal(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// An observation with all its payloads
#[derive(Clone, Debug)]
pub struct ObservationWithPayloads {
  pub observation: observation_tools_shared::Observation,
  pub payloads: Vec<StoredPayload>,
}

/// A single payload attached to an observation
#[derive(Clone, Debug)]
pub struct StoredPayload {
  pub id: PayloadId,
  pub name: String,
  pub mime_type: String,
  pub size: usize,
  pub data: PayloadData,
}

/// Whether payload data is inline or stored as a blob
#[derive(Clone, Debug)]
pub enum PayloadData {
  Inline(Vec<u8>),
  Blob,
}

/// Default page size for cursor-based pagination
pub const PAGE_SIZE: usize = 100;

/// Pagination metadata for cursor-based responses
#[derive(Clone, Debug)]
pub struct PaginationInfo {
  pub item_count: usize,
  pub previous_page_token: Option<String>,
  pub next_page_token: Option<String>,
}

/// A page of observations with pagination info
#[derive(Clone, Debug)]
pub struct ObservationPage {
  pub observations: Vec<ObservationWithPayloads>,
  pub pagination: PaginationInfo,
}

/// A page of payloads with pagination info
#[derive(Clone, Debug)]
pub struct ObservationPayloadPage {
  pub payloads: Vec<StoredPayload>,
  pub pagination: PaginationInfo,
}

/// A page of direct descendants (children) of a group
#[derive(Clone, Debug)]
pub struct GroupDirectDescendantsPage {
  pub descendants: Vec<GroupTreeNode>,
  pub pagination: PaginationInfo,
}

/// A group observation with its ancestor chain and immediate children
#[derive(Clone, Debug)]
pub struct Group {
  pub metadata: ObservationWithPayloads,
  /// Oldest ancestor first
  pub group_ancestors: Vec<GroupId>,
  pub content: GroupDirectDescendantsPage,
}

impl Group {
  pub fn group_id(&self) -> &GroupId {
    // TODO: Make this guaranteed by the API
    &self.metadata.observation.group_ids.first().expect("Group must have a group id")
  }
}

/// Result of expanding a group tree via BFS
#[derive(Clone, Debug)]
pub enum GroupTree {
  /// First level fits in max_nodes - may include deeper levels
  Tree { roots: Vec<GroupTreeNode> },
  /// First level too large - paginated list
  List(GroupDirectDescendantsPage),
}

/// A node in the group tree - either a group or a leaf observation
#[derive(Clone, Debug)]
pub enum GroupTreeNode {
  Group(Group),
  Observation(ObservationWithPayloads),
}

impl GroupTreeNode {
  pub fn group_id(&self) -> Option<&GroupId> {
    match self {
      GroupTreeNode::Group(group) => group.metadata.observation.group_ids.first(),
      _ => None,
    }
  }
}

/// Trait for storing and retrieving executions
#[async_trait::async_trait]
pub trait ExecutionStorage: Send + Sync {
  async fn store_execution(&self, execution: &Execution) -> StorageResult<()>;
  async fn get_execution(&self, id: ExecutionId) -> StorageResult<Execution>;
  async fn list_executions(
    &self,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> StorageResult<Vec<Execution>>;
  async fn count_executions(&self) -> StorageResult<usize>;
}

/// Trait for storing and retrieving observations
#[async_trait::async_trait]
pub trait ObservationStorage: Send + Sync {
  async fn store_observations(
    &self,
    observations: Vec<observation_tools_shared::Observation>,
  ) -> StorageResult<()>;

  async fn get_observation(
    &self,
    id: ObservationId,
  ) -> StorageResult<observation_tools_shared::Observation>;

  /// Paginated observations sorted by creation time (UUIDv7 order).
  /// Uses cursor-based pagination with page tokens.
  /// Optionally filters by observation type.
  async fn get_observations(
    &self,
    execution_id: ExecutionId,
    page_token: Option<String>,
    observation_type: Option<ObservationType>,
  ) -> StorageResult<ObservationPage>;
}

/// Trait for storing and retrieving payloads
#[async_trait::async_trait]
pub trait PayloadStorage: Send + Sync {
  async fn store_payloads(
    &self,
    observation_id: &ObservationId,
    payloads: &[StoredPayload],
  ) -> StorageResult<()>;

  async fn get_all_payloads(
    &self,
    observation_id: ObservationId,
  ) -> StorageResult<Vec<StoredPayload>>;

  /// Paginated payload retrieval for observation detail panel.
  /// Uses payload_id as cursor.
  async fn get_payloads(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage>;
}

/// Options for BFS group descendant expansion
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

const ROOT_SENTINEL: &str = "_ROOT_";

/// Get the GroupId key for a root, using a sentinel for the actual root.
pub fn root_group_id(root: &Option<GroupId>) -> GroupId {
  root
    .as_ref()
    .cloned()
    .unwrap_or_else(|| GroupId::from(ROOT_SENTINEL))
}

/// Assemble a nested GroupTree from a flat map of group_id → descendants.
pub fn make_group_tree(
  mut data: HashMap<GroupId, GroupDirectDescendantsPage>,
  root_group: GroupId,
) -> GroupTree {
  let Some(root_page) = data.remove(&root_group) else {
    return GroupTree::Tree { roots: Vec::new() };
  };

  if root_page.pagination.next_page_token.is_some() {
    return GroupTree::List(root_page);
  }

  let roots = root_page
    .descendants
    .into_iter()
    .map(|node| attach_children(node, &mut data))
    .collect();

  GroupTree::Tree { roots }
}

fn attach_children(
  node: GroupTreeNode,
  data: &mut HashMap<GroupId, GroupDirectDescendantsPage>,
) -> GroupTreeNode {
  match node {
    GroupTreeNode::Group(mut group) => {
      let group_id = group.metadata.observation.group_ids.first().cloned();
      if let Some(children_page) = group_id.and_then(|id| data.remove(&id)) {
        let descendants = children_page
          .descendants
          .into_iter()
          .map(|child| attach_children(child, data))
          .collect();
        group.content = GroupDirectDescendantsPage {
          descendants,
          pagination: children_page.pagination,
        };
      }
      GroupTreeNode::Group(group)
    }
    other => other,
  }
}

/// Trait for group tree operations
#[async_trait::async_trait]
pub trait GroupStorage: Send + Sync {
  async fn get_direct_descendants_page(
    &self,
    execution_id: ExecutionId,
    group_id: Option<GroupId>,
    page_token: Option<String>,
  ) -> StorageResult<GroupDirectDescendantsPage>;

  async fn get_observation_by_group_id(
    &self,
    group_id: GroupId,
  ) -> StorageResult<ObservationWithPayloads>;

  /// BFS expansion of group descendants.
  /// Returns a flat map of group_id → direct descendants page.
  /// Use `make_group_tree` to assemble into a nested `GroupTree`.
  async fn get_descendants(
    &self,
    execution_id: ExecutionId,
    options: GroupMembershipOptions,
  ) -> StorageResult<HashMap<GroupId, GroupDirectDescendantsPage>>;
}

/// Combined trait for all storage operations
pub trait MetadataStorage:
  ExecutionStorage + ObservationStorage + PayloadStorage + GroupStorage
{
}

impl<T: ExecutionStorage + ObservationStorage + PayloadStorage + GroupStorage> MetadataStorage
  for T
{
}
