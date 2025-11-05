//! Error types for the client library

use thiserror::Error;

/// Result type for client operations
pub type Result<T> = std::result::Result<T, Error>;

/// Client error types
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP client error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Shared library error
    #[error("Shared library error: {0}")]
    Shared(#[from] observation_tools_shared::Error),

    /// Channel send error
    #[error("Failed to send observation: channel closed")]
    ChannelClosed,

    /// Global execution already registered
    #[error("Global execution already registered")]
    GlobalExecutionAlreadyRegistered,

    /// No execution context available
    #[error("No execution context available")]
    NoExecutionContext,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Missing payload error
    #[error("Observation must have a payload. Use .payload() or .text_payload() to set one.")]
    MissingPayload,
}
