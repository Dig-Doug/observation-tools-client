//! API request and response types

use crate::models::Execution;
use crate::models::Observation;
use serde::Deserialize;
use serde::Serialize;

// ============================================================================
// Execution API types
// ============================================================================

/// Request to create a new execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutionRequest {
    /// The execution to create
    pub execution: Execution,
}

/// Response after creating an execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutionResponse {}

/// Query parameters for listing executions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListExecutionsQuery {
    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Number of results to skip (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    /// Search query for execution names or metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

/// Response for listing executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExecutionsResponse {
    /// List of executions
    pub executions: Vec<Execution>,

    /// Whether there are more results available
    pub has_next_page: bool,
}

/// Response for getting a single execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetExecutionResponse {
    /// The execution
    pub execution: Execution,
}

// ============================================================================
// Observation API types
// ============================================================================

/// Request to create observations (can be a batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObservationsRequest {
    /// List of observations to create
    pub observations: Vec<Observation>,
}

/// Response after creating observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObservationsResponse {}

/// Query parameters for listing observations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListObservationsQuery {
    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Number of results to skip (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    /// Search query for observation names or metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by labels (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<String>,

    /// Filter by source file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,

    /// Filter by parent span ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
}

/// Response for listing observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObservationsResponse {
    /// List of observations
    pub observations: Vec<Observation>,

    /// Whether there are more results available
    pub has_next_page: bool,
}

/// Response for getting a single observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetObservationResponse {
    /// The observation
    pub observation: Observation,
}

// ============================================================================
// Blob API types
// ============================================================================

/// Request to upload blob data for an observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBlobRequest {
    /// The blob data (base64 encoded for JSON transport)
    pub data: Vec<u8>,

    /// MIME type of the blob
    pub mime_type: String,
}

/// Response after uploading a blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBlobResponse {
    /// URL or path to access the blob
    pub url: String,

    /// Size of the uploaded blob
    pub size: usize,
}

// ============================================================================
// Error types
// ============================================================================

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,

    /// Optional error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: None,
            details: None,
        }
    }

    /// Create an error response with a code
    pub fn with_code(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: Some(code.into()),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_execution_request_serialization() {
        let execution = Execution::new("test");
        let req = CreateExecutionRequest {
            execution: execution.clone(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: CreateExecutionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(req.execution.name, deserialized.execution.name);
        assert_eq!(req.execution.id, deserialized.execution.id);
    }

    #[test]
    fn test_list_executions_query_defaults() {
        let query = ListExecutionsQuery::default();
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
        assert!(query.search.is_none());
    }

    #[test]
    fn test_error_response_creation() {
        let err = ErrorResponse::new("test error");
        assert_eq!(err.error, "test error");
        assert!(err.code.is_none());

        let err_with_code = ErrorResponse::with_code("test error", "TEST_ERROR");
        assert_eq!(err_with_code.error, "test error");
        assert_eq!(err_with_code.code, Some("TEST_ERROR".to_string()));
    }
}
