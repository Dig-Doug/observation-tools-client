//! Error types for the shared crate

use thiserror::Error;

/// Errors that can occur in the shared crate
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid execution ID format
    #[error("Invalid execution ID: {0}")]
    InvalidExecutionId(#[from] uuid::Error),

    /// Invalid observation ID format
    #[error("Invalid observation ID: {0}")]
    InvalidObservationId(uuid::Error),
}

/// Result type for shared crate operations
pub type Result<T> = std::result::Result<T, Error>;
