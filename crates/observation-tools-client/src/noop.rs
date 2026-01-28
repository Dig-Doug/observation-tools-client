//! No-op observation builder for when the `disabled` feature is enabled
//!
//! This module provides a zero-cost abstraction that discards all observation
//! data at compile time. All methods are no-ops that return immediately.

use crate::execution::ExecutionHandle;
use crate::observation_handle::SendObservation;
use observation_tools_shared::LogLevel;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::Payload;
use serde::Serialize;
use std::fmt::Debug;

/// A no-op observation builder that discards all data.
///
/// All methods are zero-cost and return immediately without doing any work.
/// This is used when the `disabled` feature is enabled to completely eliminate
/// observation overhead at compile time.
pub struct NoopObservationBuilder;

impl NoopObservationBuilder {
  /// Create a new no-op observation builder
  ///
  /// The name parameter is ignored.
  pub fn new<T: AsRef<str>>(_name: T) -> Self {
    Self
  }

  /// Set a custom observation ID (ignored)
  pub fn with_id(self, _id: ObservationId) -> Self {
    self
  }

  /// Add a label to the observation (ignored)
  pub fn label(self, _label: impl Into<String>) -> Self {
    self
  }

  /// Add multiple labels to the observation (ignored)
  pub fn labels(self, _labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
    self
  }

  /// Add metadata to the observation (ignored)
  pub fn metadata(self, _key: impl Into<String>, _value: impl Into<String>) -> Self {
    self
  }

  /// Set the source info for the observation (ignored)
  pub fn source(self, _file: impl Into<String>, _line: u32) -> Self {
    self
  }

  /// Set the parent span ID (ignored)
  pub fn parent_span_id(self, _span_id: impl Into<String>) -> Self {
    self
  }

  /// Set the observation type (ignored)
  pub fn observation_type(self, _observation_type: ObservationType) -> Self {
    self
  }

  /// Set the log level (ignored)
  pub fn log_level(self, _log_level: LogLevel) -> Self {
    self
  }

  /// Serialize the value as JSON and send the observation (no-op)
  ///
  /// The value is not serialized - this returns immediately.
  pub fn serde<T: ?Sized + Serialize + 'static>(self, _value: &T) -> SendObservation {
    SendObservation::noop()
  }

  /// Send the observation with a custom payload (no-op)
  ///
  /// The payload is discarded - this returns immediately.
  pub fn payload<T: Into<Payload>>(self, _value: T) -> SendObservation {
    SendObservation::noop()
  }

  /// Format the value using Debug and send the observation (no-op)
  ///
  /// The value is not formatted - this returns immediately.
  pub fn debug<T: Debug + ?Sized>(self, _value: &T) -> SendObservation {
    SendObservation::noop()
  }

  /// Serialize the value as JSON with explicit execution (no-op)
  pub fn serde_with_execution<T: ?Sized + Serialize + 'static>(
    self,
    _value: &T,
    _execution: &ExecutionHandle,
  ) -> SendObservation {
    SendObservation::noop()
  }

  /// Send the observation with a custom payload and explicit execution (no-op)
  pub fn payload_with_execution<T: Into<Payload>>(
    self,
    _value: T,
    _execution: &ExecutionHandle,
  ) -> SendObservation {
    SendObservation::noop()
  }

  /// Format the value using Debug with explicit execution (no-op)
  pub fn debug_with_execution<T: Debug + ?Sized>(
    self,
    _value: &T,
    _execution: &ExecutionHandle,
  ) -> SendObservation {
    SendObservation::noop()
  }
}
