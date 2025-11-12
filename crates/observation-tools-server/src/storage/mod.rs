//! Storage layer abstractions and implementations

pub mod blob;
pub mod metadata;

pub use blob::BlobStorage;
pub use blob::LocalBlobStorage;
pub use metadata::MetadataStorage;
pub use metadata::SledStorage;
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

  #[error("Search error: {0}")]
  Search(String),

  #[error("Internal error: {0}")]
  Internal(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
