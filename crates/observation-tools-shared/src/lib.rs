//! Shared types and models for observation-tools

pub mod error;
pub mod models;
mod group_id;
mod observation;
mod payload;
mod payload_id;

pub use error::Error;
pub use error::Result;
pub use group_id::GroupId;
pub use models::Execution;
pub use models::ExecutionId;
pub use observation::LogLevel;
pub use observation::Observation;
pub use observation::ObservationId;
pub use observation::ObservationType;
pub use observation::SourceInfo;
pub use payload::Markdown;
pub use payload::Payload;
pub use payload::PayloadBuilder;
pub use payload::MIME_TYPE_RUST_DEBUG;
pub use payload_id::PayloadId;

/// Payload size threshold for blob storage (64KB)
/// Payloads larger than this will be uploaded as separate blobs
pub const BLOB_THRESHOLD_BYTES: usize = 65536;

/// Number of observations to batch before uploading
pub const BATCH_SIZE: usize = 100;

/// Estimated maximum observation metadata overhead (in bytes)
/// This includes JSON structure, field names, IDs, timestamps, group_ids, etc.
/// Assumes reasonable limits:
/// - name: ~256 bytes
/// - group_ids: ~10 groups * ~40 bytes = 400 bytes
/// - metadata: ~10 entries * ~200 bytes = 2KB
/// - source info: ~256 bytes
/// - IDs, timestamps, JSON structure: ~512 bytes
pub const MAX_OBSERVATION_METADATA_OVERHEAD: usize = 4096; // 4KB

/// JSON expansion factor for byte array serialization
/// Vec<u8> serializes as [0,1,2,...] where each byte becomes ~4 characters
pub const BYTE_ARRAY_JSON_EXPANSION: usize = 4;

/// Maximum size for a single observation (payload + metadata)
/// Note: payload.data is Vec<u8> which expands ~4x when serialized as JSON
/// array
pub const MAX_OBSERVATION_SIZE: usize =
  (BLOB_THRESHOLD_BYTES * BYTE_ARRAY_JSON_EXPANSION) + MAX_OBSERVATION_METADATA_OVERHEAD;

/// Maximum size for a batch of observations
/// This is used to set the HTTP body limit for observation creation endpoints
pub const MAX_OBSERVATION_BATCH_SIZE: usize = BATCH_SIZE * MAX_OBSERVATION_SIZE;

/// Maximum size for individual blob uploads (500MB)
/// This is a generous limit for very large payloads
pub const MAX_BLOB_SIZE: usize = 500 * 1024 * 1024;

/// Payload size threshold for iframe display (5MB)
/// Payloads larger than this will be shown as a download link instead of iframe
pub const DISPLAY_THRESHOLD_BYTES: usize = 5 * 1024 * 1024;
