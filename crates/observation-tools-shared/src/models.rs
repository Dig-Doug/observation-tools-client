//! Core data models for observation-tools

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for an execution (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExecutionId(pub Uuid);

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
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an observation (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObservationId(pub Uuid);

impl ObservationId {
    /// Generate a new UUIDv7 observation ID
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parse from a string
    pub fn parse(s: &str) -> crate::Result<Self> {
        let uuid = Uuid::parse_str(s)
            .map_err(crate::Error::InvalidObservationId)?;
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
        write!(f, "{}", self.0)
    }
}

/// An execution represents the root scope for data collection.
/// All observations are associated with one execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// An observation is a single piece of collected data
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new(
        execution_id: ExecutionId,
        name: impl Into<String>,
        payload: Payload,
    ) -> Self {
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
}
