//! Core data models for observation-tools

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for an execution (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9a")]
pub struct ExecutionId(Uuid);

impl Serialize for ExecutionId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl ExecutionId {
  /// Generate a new UUIDv7 execution ID
  pub fn new() -> Self {
    Self(Uuid::now_v7())
  }

  /// Create a nil (all zeros) execution ID for placeholder use
  pub fn nil() -> Self {
    Self(Uuid::nil())
  }

  /// Parse from a string
  pub fn parse(s: &str) -> crate::Result<Self> {
    Ok(Self(Uuid::parse_str(s)?))
  }
}

impl Default for ExecutionId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for ExecutionId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.as_simple().to_string())
  }
}

/// An execution represents the root scope for data collection.
/// All observations are associated with one execution.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Execution {
  /// Unique identifier for this execution
  pub id: ExecutionId,

  /// User-defined name for this execution
  pub name: String,

  /// User-defined metadata as key-value pairs
  #[serde(default)]
  pub metadata: HashMap<String, String>,

  /// When this execution was created
  pub created_at: DateTime<Utc>,

  /// When this execution was last updated
  pub updated_at: DateTime<Utc>,
}

impl Execution {
  /// Create a new execution with the given name
  pub fn new(name: impl Into<String>) -> Self {
    let now = Utc::now();
    Self {
      id: ExecutionId::new(),
      name: name.into(),
      metadata: HashMap::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Create a new execution with a specific ID (for testing only)
  ///
  /// This allows tests to create an execution with a known ID before
  /// sending it to the server.
  #[cfg(any(test, feature = "testing"))]
  pub fn with_id(id: ExecutionId, name: impl Into<String>) -> Self {
    let now = Utc::now();
    Self {
      id,
      name: name.into(),
      metadata: HashMap::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Create a new execution with metadata
  pub fn with_metadata(name: impl Into<String>, metadata: HashMap<String, String>) -> Self {
    let now = Utc::now();
    Self {
      id: ExecutionId::new(),
      name: name.into(),
      metadata,
      created_at: now,
      updated_at: now,
    }
  }
}
