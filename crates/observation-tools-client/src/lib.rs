//! Client library for observation-tools

mod client;
pub(crate) mod context;
mod error;
mod execution;
mod observation;
pub mod server_client;

pub use client::Client;
pub use client::ClientBuilder;
pub use client::BATCH_SIZE;
pub use client::BLOB_THRESHOLD_BYTES;
pub use context::with_execution;
pub use error::Error;
pub use error::Result;
pub use execution::BeginExecution;
pub use execution::ExecutionHandle;
pub use execution::SendObservation;
pub use observation::ObservationBuilder;
pub use observation_tools_shared::IntoCustomPayload;
// Re-export from shared for convenience
pub use observation_tools_shared::IntoPayload;
pub use observation_tools_shared::Payload;

// Re-export procedural macro
pub use observation_tools_macros::observe;

/// Register a global execution shared across all threads
///
/// This replaces any previously set execution context.
/// The execution context is shared across all threads in the process,
/// making it accessible from any thread via `current_execution()` or the
/// `observe!` macro.
///
/// Note: This is a process-wide global. If you need per-request or per-task
/// isolation, consider passing the `ExecutionHandle` explicitly instead of
/// using the global context.
pub fn register_global_execution(execution: ExecutionHandle) -> Result<()> {
  context::set_global_execution(execution)
}

/// Get the current execution from the global context
///
/// This returns a clone of the execution handle that is shared across all
/// threads.
pub fn current_execution() -> Option<ExecutionHandle> {
  context::get_current_execution()
}

/// Clear the global execution context
///
/// This clears the execution context that is shared across all threads.
pub fn clear_global_execution() {
  context::clear_global_execution()
}
