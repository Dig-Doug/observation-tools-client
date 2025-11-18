//! Client library for observation-tools
//!
//! This crate provides the client-side API for instrumenting code
//! and logging observations.
//!
//! # Custom Serialization with IntoPayload
//!
//! The `IntoPayload` trait allows you to control how your types are serialized
//! into observation payloads. This is useful for:
//!
//! - Serializing primitives as `text/plain` instead of JSON
//! - Providing custom serialization logic for complex types
//! - Optimizing payload size and format
//! - Supporting binary or custom MIME types
//!
//! ## Built-in Implementations
//!
//! The following types have built-in `IntoPayload` implementations that serialize
//! as `text/plain`:
//!
//! - String types: `String`, `&str`, `&String`
//! - Integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
//! - Floats: `f32`, `f64`
//! - `bool`, `char`
//!
//! ## Using with the `observe!` Macro
//!
//! The `observe!` macro uses `custom_payload()` by default, which means it
//! automatically uses the `IntoPayload` trait:
//!
//! ```no_run
//! use observation_tools_client::observe;
//!
//! // Strings are serialized as text/plain
//! observe!("message", "Hello, world!")?;
//!
//! // Numbers are serialized as text/plain
//! observe!("count", 42)?;
//! # Ok::<(), observation_tools_client::Error>(())
//! ```
//!
//! ## Using with the Builder API
//!
//! You can use either `custom_payload()` for `IntoPayload` types or `payload()`
//! for serde JSON serialization:
//!
//! ```no_run
//! use observation_tools_client::ObservationBuilder;
//!
//! // Using IntoPayload (text/plain)
//! ObservationBuilder::new("count")
//!     .custom_payload(42)
//!     .build()?;
//!
//! // Using serde JSON (application/json)
//! #[derive(serde::Serialize)]
//! struct MyData {
//!     value: String,
//! }
//!
//! let data = MyData { value: "test".to_string() };
//! ObservationBuilder::new("custom_data")
//!     .payload(data)
//!     .build()?;
//! # Ok::<(), observation_tools_client::Error>(())
//! ```
//!
//! ## Implementing IntoPayload for Your Types
//!
//! You can implement `IntoPayload` for your own types to customize serialization:
//!
//! ```no_run
//! use observation_tools_client::{IntoPayload, Payload};
//!
//! struct UserId(u64);
//!
//! impl IntoPayload for UserId {
//!     fn into_payload(self) -> Payload {
//!         Payload::text(format!("user_{}", self.0))
//!     }
//! }
//!
//! // Now you can use it with observe!
//! # use observation_tools_client::observe;
//! let user_id = UserId(123);
//! observe!("user_id", user_id)?;
//! # Ok::<(), observation_tools_client::Error>(())
//! ```
//!
//! ## Why No Blanket Implementation?
//!
//! This crate intentionally does NOT provide a blanket implementation for all
//! `T: Serialize` types:
//!
//! ```ignore
//! // This is NOT provided:
//! impl<T: Serialize> IntoPayload for T { ... }
//! ```
//!
//! This is because Rust's orphan rules would prevent you from implementing
//! both `Serialize` and `IntoPayload` for your own types, which would create
//! conflicts. By not providing a blanket implementation, you have the flexibility
//! to implement both traits on your types.

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

// Re-export from shared for convenience
pub use observation_tools_shared::IntoPayload;
pub use observation_tools_shared::Payload;

// Re-export macros
#[macro_export]
macro_rules! observe {
    // Simple case: observe!("name", value)
    // Uses custom_payload which works with IntoPayload trait
    ($name:expr, $value:expr) => {{
        $crate::ObservationBuilder::new($name)
            .custom_payload($value)
            .source(file!(), line!())
            .build()
    }};

    // Structured case with custom payload: observe!(name = "...", label = "...", payload = ...)
    // Uses custom_payload for IntoPayload types
    (name = $name:expr, label = $label:expr, payload = $payload:expr) => {{
        $crate::ObservationBuilder::new($name)
            .label($label)
            .custom_payload($payload)
            .source(file!(), line!())
            .build()
    }};

    // Explicit JSON serialization: observe!(name = "...", json = ...)
    // Uses serde JSON serialization
    (name = $name:expr, json = $json_value:expr) => {{
        $crate::ObservationBuilder::new($name)
            .payload($json_value)
            .source(file!(), line!())
            .build()
    }};

    // With metadata: observe!(name = "...", payload = ..., metadata = {key: val, ...})
    (name = $name:expr, payload = $payload:expr, metadata = { $($key:expr => $val:expr),* }) => {{
        let mut builder = $crate::ObservationBuilder::new($name)
            .custom_payload($payload)
            .source(file!(), line!());
        $(
            builder = builder.metadata($key, $val);
        )*
        builder.build()
    }};
}

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
