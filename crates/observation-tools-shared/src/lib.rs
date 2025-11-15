//! Shared types and models for observation-tools

pub mod error;
pub mod models;

pub use error::Error;
pub use error::Result;
pub use models::Execution;
pub use models::ExecutionId;
pub use models::Observation;
pub use models::ObservationId;
