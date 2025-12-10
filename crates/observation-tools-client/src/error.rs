//! Error types for the client library

use crate::client::UploaderMessage;
use async_channel::TrySendError;
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

  /// No execution context available
  #[error("No execution context available")]
  NoExecutionContext,

  /// Configuration error
  #[error("Configuration error: {0}")]
  Config(String),

  /// Missing payload error
  #[error("Observation must have a payload. Use .payload() or .text_payload() to set one.")]
  MissingPayload,

  #[error("Failed to send uploader message: {0}")]
  TrySendError(String),

  /// Upload failed with error
  #[error("Upload failed: {0}")]
  UploadFailed(String),

  #[error("Creation error")]
  CreationError,
}

impl From<TrySendError<UploaderMessage>> for Error {
  fn from(err: TrySendError<UploaderMessage>) -> Self {
    Error::TrySendError(err.to_string())
  }
}

impl From<Error> for napi::Error {
  fn from(err: Error) -> Self {
    napi::Error::from_reason(err.to_string())
  }
}
