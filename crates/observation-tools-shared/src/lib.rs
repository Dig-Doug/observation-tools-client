//! Shared types and models for observation-tools

pub mod api;
pub mod error;
pub mod models;

pub use error::{Error, Result};
pub use models::{Execution, ExecutionId, Observation, ObservationId};
