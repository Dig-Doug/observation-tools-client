//! Logger implementation that captures logs as observations
//!
//! This module provides a `log::Log` implementation that automatically
//! captures log records and creates observations using the active execution context.

use crate::context;
use crate::observation::ObservationBuilder;
use log::{Level, Log, Metadata, Record};
use serde_json::json;

/// A logger implementation that captures logs as observations
///
/// This logger integrates with Rust's `log` crate and automatically creates
/// observations for log records. Each log record is converted into an observation
/// with the following properties:
///
/// - **Name**: The log target (e.g., module path)
/// - **Label**: "log/{level}" (e.g., "log/info", "log/error")
/// - **Payload**: JSON object containing the log message and metadata
/// - **Source**: File and line number where the log was recorded
/// - **Metadata**: Log level and target
///
/// # Example
///
/// ```rust,ignore
/// use observation_tools_client::{ClientBuilder, ObservationLogger};
///
/// // Initialize the logger
/// ObservationLogger::init().expect("Failed to initialize logger");
///
/// // Create a client and execution
/// let client = ClientBuilder::new()
///     .base_url("http://localhost:3000")
///     .build()
///     .expect("Failed to create client");
///
/// let execution = client
///     .begin_execution("my-execution")
///     .expect("Failed to begin execution")
///     .wait_for_upload()
///     .await
///     .expect("Failed to create execution");
///
/// // Register the execution as global context
/// observation_tools_client::register_global_execution(execution)
///     .expect("Failed to register execution");
///
/// // Now all log calls will be captured as observations
/// log::info!("This is an info message");
/// log::error!("This is an error message");
/// ```
pub struct ObservationLogger;

impl ObservationLogger {
  /// Create a new ObservationLogger
  pub const fn new() -> Self {
    Self
  }

  /// Initialize the ObservationLogger as the global logger
  ///
  /// This sets the ObservationLogger as the global logger for the `log` crate,
  /// with a default maximum level of `Info`.
  ///
  /// # Errors
  ///
  /// Returns an error if a logger has already been set.
  pub fn init() -> Result<(), log::SetLoggerError> {
    Self::init_with_level(Level::Info)
  }

  /// Initialize the ObservationLogger with a specific maximum log level
  ///
  /// # Errors
  ///
  /// Returns an error if a logger has already been set.
  pub fn init_with_level(max_level: Level) -> Result<(), log::SetLoggerError> {
    // Use a static logger instance
    static LOGGER: ObservationLogger = ObservationLogger;

    log::set_logger(&LOGGER)?;
    log::set_max_level(max_level.to_level_filter());
    Ok(())
  }
}

impl Log for ObservationLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= log::max_level()
  }

  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    // Only capture logs if there's an active execution context
    if context::get_current_execution().is_none() {
      return;
    }

    let level = record.level();
    let target = record.target();
    let message = format!("{}", record.args());

    // Create the payload with log details
    let payload = json!({
      "message": message,
      "level": level.to_string().to_lowercase(),
      "target": target,
    });

    // Build the observation
    let mut builder = ObservationBuilder::new(target)
      .label(format!("log/{}", level.to_string().to_lowercase()))
      .payload(payload)
      .metadata("log_level", level.to_string().to_lowercase())
      .metadata("log_target", target);

    // Add source information if available
    if let (Some(file), Some(line)) = (record.file(), record.line()) {
      builder = builder.source(file, line);
    }

    // Build and send the observation (errors are logged internally)
    let _ = builder.build();
  }

  fn flush(&self) {
    // No buffering, so nothing to flush
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_logger_creation() {
    let logger = ObservationLogger::new();
    assert!(std::mem::size_of_val(&logger) == 0); // Zero-sized type
  }
}
