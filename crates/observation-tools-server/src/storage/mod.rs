//! Storage layer abstractions and implementations

pub mod blob;
pub mod metadata;
pub mod proto;

pub use blob::BlobStorage;
pub use blob::LocalBlobStorage;
pub use metadata::MetadataStorage;
pub use metadata::SledStorage;
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
