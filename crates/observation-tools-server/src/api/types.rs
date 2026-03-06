//! API request and response types

use observation_tools_shared::models::Execution;
use observation_tools_shared::Observation;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

// ============================================================================
// Execution API types
// ============================================================================

/// Request to create a new execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateExecutionRequest {
  /// The execution to create
  pub execution: Execution,
}

/// Response after creating an execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateExecutionResponse {}

/// Query parameters for listing executions
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, IntoParams)]
pub struct ListExecutionsQuery {
  /// Maximum number of results to return
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<usize>,

  /// Number of results to skip (for pagination)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset: Option<usize>,
}

/// Response for listing executions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListExecutionsResponse {
  /// List of executions
  pub executions: Vec<Execution>,

  /// Whether there are more results available
  pub has_next_page: bool,
}

/// Response for getting a single execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetExecutionResponse {
  /// The execution
  pub execution: Execution,
}

// ============================================================================
// Observation API types
// ============================================================================

/// Request to create observations (can be a batch)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateObservationsRequest {
  /// List of observations to create
  pub observations: Vec<Observation>,
}

/// Response after creating observations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateObservationsResponse {}

/// Query parameters for listing observations (cursor-based pagination)
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, IntoParams)]
pub struct ListObservationsQuery {
  /// Cursor token for the next page
  #[serde(skip_serializing_if = "Option::is_none")]
  pub page_token: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
