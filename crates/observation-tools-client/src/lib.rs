//! Client library for observation-tools
//!
//! This crate provides the client-side API for instrumenting code
//! and logging observations.

mod client;
pub(crate) mod context;
mod error;
mod execution;
mod observation;

pub use client::Client;
pub use client::ClientBuilder;
pub use error::Error;
pub use error::Result;
pub use execution::ExecutionHandle;
pub use observation::ObservationBuilder;

// Re-export macros
#[macro_export]
macro_rules! observe {
    // Simple case: observe!("name", value)
    ($name:expr, $value:expr) => {{
        $crate::ObservationBuilder::new($name)
            .payload($value)
            .source(file!(), line!())
            .build()
    }};

    // Structured case: observe!(name = "...", label = "...", payload = ...)
    (name = $name:expr, label = $label:expr, payload = $payload:expr) => {{
        $crate::ObservationBuilder::new($name)
            .label($label)
            .payload($payload)
            .source(file!(), line!())
            .build()
    }};

    // With metadata: observe!(name = "...", payload = ..., metadata = {key: val, ...})
    (name = $name:expr, payload = $payload:expr, metadata = { $($key:expr => $val:expr),* }) => {{
        let mut builder = $crate::ObservationBuilder::new($name)
            .payload($payload)
            .source(file!(), line!());
        $(
            builder = builder.metadata($key, $val);
        )*
        builder.build()
    }};
}

/// Register a global execution for the current thread
pub fn register_global_execution(execution: ExecutionHandle) -> Result<()> {
    context::set_global_execution(execution)
}

/// Get the current execution from context
pub fn current_execution() -> Option<ExecutionHandle> {
    context::get_current_execution()
}
