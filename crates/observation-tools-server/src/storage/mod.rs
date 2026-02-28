//! Storage layer abstractions and implementations

pub mod blob;
pub mod metadata;
pub mod proto;

pub use blob::BlobStorage;
pub use blob::LocalBlobStorage;
pub use metadata::MetadataStorage;
pub use metadata::SledStorage;
use observation_tools_shared::GroupId;
use observation_tools_shared::PayloadId;
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
  Database(#[from] sled::Error),

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
