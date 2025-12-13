//! Core data models for observation-tools

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
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

  /// Create a nil (all zeros) observation ID for placeholder use
  pub fn nil() -> Self {
    Self(Uuid::nil())
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
  pub data: Vec<u8>,

  /// MIME type of the payload (e.g., "text/plain", "application/json")
  pub mime_type: String,

  /// Size of the payload in bytes
  pub size: usize,
}

impl Payload {
  /// Create a new payload from text
  pub fn text(data: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: "text/plain".to_string(),
      size,
    }
  }

  /// Create a new payload from JSON
  pub fn json(data: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: "application/json".to_string(),
      size,
    }
  }

  /// Create a new payload with a custom MIME type
  pub fn with_mime_type(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: mime_type.into(),
      size,
    }
  }

  /// Get data as UTF-8 string (for testing). Panics if not valid UTF-8.
  #[cfg(any(test, feature = "testing"))]
  pub fn data_as_str(&self) -> &str {
    std::str::from_utf8(&self.data).expect("payload data is not valid UTF-8")
  }
}

/// Type of observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ObservationType {
  LogEntry,
  Payload,
}

/// Log level for observations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warning,
  Error,
}

impl From<log::Level> for LogLevel {
  fn from(level: log::Level) -> Self {
    match level {
      log::Level::Trace => LogLevel::Trace,
      log::Level::Debug => LogLevel::Debug,
      log::Level::Info => LogLevel::Info,
      log::Level::Warn => LogLevel::Warning,
      log::Level::Error => LogLevel::Error,
    }
  }
}

/// Trait for types that can be converted into an observation payload.
pub trait IntoPayload {
  /// Convert this value into a payload
  fn to_payload(&self) -> Payload;
}

impl IntoPayload for str {
  fn to_payload(&self) -> Payload {
    Payload::text(self.to_string())
  }
}

impl<T> IntoPayload for T
where
  T: Serialize + 'static,
{
  fn to_payload(&self) -> Payload {
    if let Some(string_ref) = (self as &dyn Any).downcast_ref::<String>() {
      Payload::text(string_ref.clone())
    } else {
      let json = serde_json::to_string(self).unwrap_or_default();
      Payload::json(json)
    }
  }
}

/// Implement IntoPayload for custom types if Serde serialization is not
/// sufficient..
pub trait IntoCustomPayload {
  /// Convert this value into a payload
  fn to_payload(&self) -> Payload;
}

/// A wrapper type for markdown content.
///
/// Use this to create observations with markdown payloads that will be
/// rendered as HTML in the UI.
///
/// # Example
/// ```rust
/// use observation_tools_shared::models::Markdown;
///
/// let md = Markdown::from("# Hello\n\nThis is **bold** text.");
/// ```
#[derive(Debug, Clone)]
pub struct Markdown {
  content: String,
}

impl Markdown {
  /// Create a new Markdown payload from any type that can be converted to a
  /// String.
  pub fn from(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
    }
  }
}

impl IntoCustomPayload for Markdown {
  fn to_payload(&self) -> Payload {
    Payload::with_mime_type(self.content.clone(), "text/markdown")
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

  /// Type of observation
  pub observation_type: ObservationType,

  /// Log level for this observation
  pub log_level: LogLevel,

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
  pub fn new(
    execution_id: ExecutionId,
    name: impl Into<String>,
    payload: Payload,
    observation_type: ObservationType,
    log_level: LogLevel,
  ) -> Self {
    Self {
      id: ObservationId::new(),
      execution_id,
      name: name.into(),
      observation_type,
      log_level,
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
