//! Execution context propagation

use crate::error::Result;
use crate::execution::ExecutionHandle;
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::sync::RwLock;
use tokio::task_local;

// Task-local execution context (per-task isolation)
task_local! {
  static TASK_EXECUTION: ExecutionHandle;
}

// Global execution context (process-wide, shared across all tasks)
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
/// This checks task-local storage first, then falls back to the global context.
/// Returns a clone of the execution handle.
pub(crate) fn get_current_execution() -> Option<ExecutionHandle> {
  // Try task-local storage first
  if let Ok(handle) = TASK_EXECUTION.try_with(|h| h.clone()) {
    return Some(handle);
  }

  // Fall back to global context
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

/// Run a future with a task-local execution context
///
/// This sets the execution context for the duration of the future,
/// providing task-level isolation. This is useful for concurrent tasks
/// that need separate execution contexts.
///
/// # Example
///
/// ```rust,ignore
/// let execution1 = client.begin_execution("task-1")?.wait_for_upload().await?;
/// let execution2 = client.begin_execution("task-2")?.wait_for_upload().await?;
///
/// // Run two tasks concurrently with different execution contexts
/// let (result1, result2) = tokio::join!(
///   with_execution(execution1, async {
///     observe!("observation-1").serde(&"data from task 1");
///     Ok::<_, Error>(())
///   }),
///   with_execution(execution2, async {
///     observe!("observation-2").serde(&"data from task 2");
///     Ok::<_, Error>(())
///   })
/// );
/// ```
pub async fn with_execution<F, T>(execution: ExecutionHandle, future: F) -> T
where
  F: std::future::Future<Output = T>,
{
  TASK_EXECUTION.scope(execution, future).await
}

/// Get the current tracing span ID as a string.
///
/// Returns the ID of the currently active tracing span, or `None` if there is
/// no active span. This is used for automatically attributing observations
/// to the current span when the `tracing` feature is enabled.
#[cfg(feature = "tracing")]
pub(crate) fn get_current_tracing_span_id() -> Option<String> {
  let current = tracing::Span::current();
  current.id().map(|id| id.into_u64().to_string())
}

/// Extension trait for propagating observation context to spawned async tasks.
///
/// This trait allows you to easily attach the current execution context to
/// futures that will be spawned with `tokio::spawn` or similar.
///
/// # Example
///
/// ```rust,ignore
/// use observation_tools::WithObservations;
///
/// // The spawned task will inherit the current execution context
/// tokio::spawn(async move {
///     observe!("spawned-task").serde(&"data from spawned task");
///     Ok::<_, Error>(())
/// }.with_observations());
/// ```
pub trait WithObservations: Future + Sized {
  /// Attach the current execution context to this future.
  ///
  /// If there is a current execution context (either task-local or global),
  /// the returned future will run with that context. If there is no current
  /// execution context, the future runs unchanged.
  fn with_observations(self) -> WithObservationsFuture<Self::Output>;
}

impl<F: Future + Send + 'static> WithObservations for F {
  fn with_observations(self) -> WithObservationsFuture<Self::Output> {
    match get_current_execution() {
      Some(execution) => WithObservationsFuture(Box::pin(TASK_EXECUTION.scope(execution, self))),
      None => WithObservationsFuture(Box::pin(self)),
    }
  }
}

/// Future wrapper that propagates execution context.
pub struct WithObservationsFuture<T>(Pin<Box<dyn Future<Output = T> + Send>>);

impl<T> Future for WithObservationsFuture<T> {
  type Output = T;

  fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<T> {
    self.0.as_mut().poll(cx)
  }
}
