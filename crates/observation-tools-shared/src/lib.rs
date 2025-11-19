//! Shared types and models for observation-tools

pub mod error;
pub mod models;

pub use error::Error;
pub use error::Result;
pub use models::Execution;
pub use models::ExecutionId;
pub use models::IntoCustomPayload;
pub use models::IntoPayload;
pub use models::Observation;
pub use models::ObservationId;
pub use models::Payload;

/// Payload size threshold for blob storage (64KB)
/// Payloads larger than this will be uploaded as separate blobs
pub const BLOB_THRESHOLD_BYTES: usize = 65536;

/// Number of observations to batch before uploading
pub const BATCH_SIZE: usize = 100;

/// Estimated maximum observation metadata overhead (in bytes)
/// This includes JSON structure, field names, IDs, timestamps, labels, etc.
/// Assumes reasonable limits:
/// - name: ~256 bytes
/// - labels: ~10 labels * ~100 bytes = 1KB
/// - metadata: ~10 entries * ~200 bytes = 2KB
/// - source info: ~256 bytes
/// - IDs, timestamps, JSON structure: ~512 bytes
pub const MAX_OBSERVATION_METADATA_OVERHEAD: usize = 4096; // 4KB

/// Maximum size for a single observation (payload + metadata)
pub const MAX_OBSERVATION_SIZE: usize = BLOB_THRESHOLD_BYTES + MAX_OBSERVATION_METADATA_OVERHEAD;

/// Maximum size for a batch of observations
/// This is used to set the HTTP body limit for observation creation endpoints
pub const MAX_OBSERVATION_BATCH_SIZE: usize = BATCH_SIZE * MAX_OBSERVATION_SIZE;

/// Maximum size for individual blob uploads (500MB)
/// This is a generous limit for very large payloads
pub const MAX_BLOB_SIZE: usize = 500 * 1024 * 1024;

/// Payload size threshold for iframe display (5MB)
/// Payloads larger than this will be shown as a download link instead of iframe
pub const DISPLAY_THRESHOLD_BYTES: usize = 5 * 1024 * 1024;
