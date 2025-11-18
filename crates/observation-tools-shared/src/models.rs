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

/// Unique identifier for an observation (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9b")]
pub struct ObservationId(Uuid);

impl Serialize for ObservationId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl ObservationId {
  /// Generate a new UUIDv7 observation ID
  pub fn new() -> Self {
    Self(Uuid::now_v7())
  }

  /// Parse from a string
  pub fn parse(s: &str) -> crate::Result<Self> {
    let uuid = Uuid::parse_str(s).map_err(crate::Error::InvalidObservationId)?;
    Ok(Self(uuid))
  }
}

impl Default for ObservationId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for ObservationId {
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

/// Source location information for an observation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SourceInfo {
  /// Source file path
  pub file: String,

  /// Line number in the source file
  pub line: u32,

  /// Optional column number
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<u32>,
}

/// Payload data for an observation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Payload {
  /// The actual payload data
  pub data: String,

  /// MIME type of the payload (e.g., "text/plain", "application/json")
  pub mime_type: String,

  /// Size of the payload in bytes
  pub size: usize,
}

impl Payload {
  /// Create a new payload from text
  pub fn text(data: impl Into<String>) -> Self {
    let data = data.into();
    let size = data.len();
    Self {
      data,
      mime_type: "text/plain".to_string(),
      size,
    }
  }

  /// Create a new payload from JSON
  pub fn json(data: impl Into<String>) -> Self {
    let data = data.into();
    let size = data.len();
    Self {
      data,
      mime_type: "application/json".to_string(),
      size,
    }
  }

  /// Create a new payload with a custom MIME type
  pub fn with_mime_type(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
    let data = data.into();
    let size = data.len();
    Self {
      data,
      mime_type: mime_type.into(),
      size,
    }
  }
}

/// Trait for types that can be converted into an observation payload.
///
/// This trait allows custom types to define their own serialization logic
/// for observations. Types implementing this trait can provide optimized
/// or custom payload representations instead of relying on JSON serialization.
///
/// # When to Implement
///
/// Implement this trait when you want to:
/// - Serialize primitive types as text instead of JSON
/// - Provide custom binary representations
/// - Optimize serialization for specific types
/// - Control the MIME type of the payload
///
/// # Examples
///
/// ```
/// use observation_tools_shared::models::{IntoPayload, Payload};
///
/// struct CustomData {
///     value: String,
/// }
///
/// impl IntoPayload for CustomData {
///     fn into_payload(self) -> Payload {
///         Payload::text(self.value)
///     }
/// }
/// ```
///
/// # Note on Blanket Implementations
///
/// This crate intentionally does NOT provide a blanket implementation
/// for all `T: Serialize` types. This allows you to implement both
/// `Serialize` and `IntoPayload` for your types without conflicts.
pub trait IntoPayload {
  /// Convert this value into a payload
  fn into_payload(self) -> Payload;
}

// Implement IntoPayload for Payload itself (identity conversion)
impl IntoPayload for Payload {
  fn into_payload(self) -> Payload {
    self
  }
}

// String types - serialize as text/plain
impl IntoPayload for String {
  fn into_payload(self) -> Payload {
    Payload::text(self)
  }
}

impl IntoPayload for &str {
  fn into_payload(self) -> Payload {
    Payload::text(self)
  }
}

impl IntoPayload for &String {
  fn into_payload(self) -> Payload {
    Payload::text(self)
  }
}

// Numeric types - serialize as text/plain
macro_rules! impl_into_payload_for_int {
  ($($t:ty),*) => {
    $(
      impl IntoPayload for $t {
        fn into_payload(self) -> Payload {
          Payload::text(self.to_string())
        }
      }
    )*
  };
}

impl_into_payload_for_int!(i8, i16, i32, i64, i128, isize);
impl_into_payload_for_int!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_into_payload_for_float {
  ($($t:ty),*) => {
    $(
      impl IntoPayload for $t {
        fn into_payload(self) -> Payload {
          Payload::text(self.to_string())
        }
      }
    )*
  };
}

impl_into_payload_for_float!(f32, f64);

// Boolean - serialize as text/plain
impl IntoPayload for bool {
  fn into_payload(self) -> Payload {
    Payload::text(self.to_string())
  }
}

// Char - serialize as text/plain
impl IntoPayload for char {
  fn into_payload(self) -> Payload {
    Payload::text(self.to_string())
  }
}

/// An observation is a single piece of collected data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Observation {
  /// Unique identifier for this observation
  pub id: ObservationId,

  /// ID of the execution this observation belongs to
  pub execution_id: ExecutionId,

  /// User-defined name for this observation
  pub name: String,

  /// Source location where this observation was created
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<SourceInfo>,

  /// The observation payload
  pub payload: Payload,

  /// User-defined metadata as key-value pairs
  #[serde(default)]
  pub metadata: HashMap<String, String>,

  /// Hierarchical labels for grouping observations
  /// Uses path convention (e.g., "api/request/headers")
  #[serde(default)]
  pub labels: Vec<String>,

  /// Parent span ID (for tracing integration)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent_span_id: Option<String>,

  /// When this observation was created
  pub created_at: DateTime<Utc>,
}

impl Observation {
  /// Create a new observation
  pub fn new(execution_id: ExecutionId, name: impl Into<String>, payload: Payload) -> Self {
    Self {
      id: ObservationId::new(),
      execution_id,
      name: name.into(),
      source: None,
      payload,
      metadata: HashMap::new(),
      labels: Vec::new(),
      parent_span_id: None,
      created_at: Utc::now(),
    }
  }

  /// Set the source information
  pub fn with_source(mut self, source: SourceInfo) -> Self {
    self.source = Some(source);
    self
  }

  /// Set metadata
  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  /// Add a label
  pub fn with_label(mut self, label: impl Into<String>) -> Self {
    self.labels.push(label.into());
    self
  }

  /// Set labels
  pub fn with_labels(mut self, labels: Vec<String>) -> Self {
    self.labels = labels;
    self
  }

  /// Set parent span ID
  pub fn with_parent_span(mut self, span_id: impl Into<String>) -> Self {
    self.parent_span_id = Some(span_id.into());
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_execution_id_generation() {
    let id1 = ExecutionId::new();
    let id2 = ExecutionId::new();
    assert_ne!(id1, id2);
  }

  #[test]
  fn test_observation_id_generation() {
    let id1 = ObservationId::new();
    let id2 = ObservationId::new();
    assert_ne!(id1, id2);
  }

  #[test]
  fn test_execution_creation() {
    let exec = Execution::new("test_execution");
    assert_eq!(exec.name, "test_execution");
    assert!(exec.metadata.is_empty());
  }

  #[test]
  fn test_observation_creation() {
    let exec_id = ExecutionId::new();
    let payload = Payload::text("test data");
    let obs = Observation::new(exec_id, "test_obs", payload);

    assert_eq!(obs.name, "test_obs");
    assert_eq!(obs.execution_id, exec_id);
    assert_eq!(obs.payload.mime_type, "text/plain");
  }

  #[test]
  fn test_observation_with_labels() {
    let exec_id = ExecutionId::new();
    let payload = Payload::json(r#"{"key": "value"}"#);
    let obs = Observation::new(exec_id, "test", payload)
      .with_label("api/request")
      .with_label("http");

    assert_eq!(obs.labels.len(), 2);
    assert_eq!(obs.payload.mime_type, "application/json");
  }

  #[test]
  fn test_into_payload_string() {
    let payload = "Hello, world!".into_payload();
    assert_eq!(payload.mime_type, "text/plain");
    assert_eq!(payload.data, "Hello, world!");
    assert_eq!(payload.size, "Hello, world!".len());
  }

  #[test]
  fn test_into_payload_owned_string() {
    let s = String::from("Test string");
    let payload = s.into_payload();
    assert_eq!(payload.mime_type, "text/plain");
    assert_eq!(payload.data, "Test string");
  }

  #[test]
  fn test_into_payload_integers() {
    let p1 = 42i32.into_payload();
    assert_eq!(p1.mime_type, "text/plain");
    assert_eq!(p1.data, "42");

    let p2 = 100u64.into_payload();
    assert_eq!(p2.mime_type, "text/plain");
    assert_eq!(p2.data, "100");

    let p3 = (-123i128).into_payload();
    assert_eq!(p3.mime_type, "text/plain");
    assert_eq!(p3.data, "-123");
  }

  #[test]
  fn test_into_payload_floats() {
    let p1 = 3.14f32.into_payload();
    assert_eq!(p1.mime_type, "text/plain");
    assert_eq!(p1.data, "3.14");

    let p2 = 2.718f64.into_payload();
    assert_eq!(p2.mime_type, "text/plain");
    assert_eq!(p2.data, "2.718");
  }

  #[test]
  fn test_into_payload_bool() {
    let p1 = true.into_payload();
    assert_eq!(p1.mime_type, "text/plain");
    assert_eq!(p1.data, "true");

    let p2 = false.into_payload();
    assert_eq!(p2.mime_type, "text/plain");
    assert_eq!(p2.data, "false");
  }

  #[test]
  fn test_into_payload_char() {
    let p = 'x'.into_payload();
    assert_eq!(p.mime_type, "text/plain");
    assert_eq!(p.data, "x");
  }

  #[test]
  fn test_into_payload_payload_identity() {
    let original = Payload::json(r#"{"test": "data"}"#);
    let payload = original.clone().into_payload();
    assert_eq!(payload.mime_type, "application/json");
    assert_eq!(payload.data, r#"{"test": "data"}"#);
  }

  #[test]
  fn test_custom_into_payload_implementation() {
    struct CustomType {
      value: String,
    }

    impl IntoPayload for CustomType {
      fn into_payload(self) -> Payload {
        Payload::with_mime_type(format!("custom: {}", self.value), "text/custom")
      }
    }

    let custom = CustomType {
      value: "test".to_string(),
    };
    let payload = custom.into_payload();
    assert_eq!(payload.mime_type, "text/custom");
    assert_eq!(payload.data, "custom: test");
  }
}
