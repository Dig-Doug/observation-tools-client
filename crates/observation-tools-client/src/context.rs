//! Execution context propagation

use crate::error::Result;
use crate::execution::ExecutionHandle;
use std::sync::{OnceLock, RwLock};

static EXECUTION_CONTEXT: OnceLock<RwLock<Option<ExecutionHandle>>> = OnceLock::new();

fn get_context() -> &'static RwLock<Option<ExecutionHandle>> {
  EXECUTION_CONTEXT.get_or_init(|| RwLock::new(None))
}

/// Set the global execution shared across all threads
///
/// This replaces any previously set execution context.
/// The execution context is shared across all threads in the process.
pub(crate) fn set_global_execution(execution: ExecutionHandle) -> Result<()> {
  let context = get_context();
  let mut ctx = context.write().expect("Failed to acquire write lock");
  *ctx = Some(execution);
  Ok(())
}

/// Get the current execution from context
///
/// This returns a clone of the execution handle that is shared across all threads.
pub(crate) fn get_current_execution() -> Option<ExecutionHandle> {
  let context = get_context();
  let ctx = context.read().expect("Failed to acquire read lock");
  ctx.clone()
}

/// Clear the global execution context
///
/// This clears the execution context that is shared across all threads.
pub(crate) fn clear_global_execution() {
  let context = get_context();
  let mut ctx = context.write().expect("Failed to acquire write lock");
  *ctx = None;
}
