use std::time::Instant;

/// Data stored in span extensions for later use on span close
#[derive(Debug)]
pub(crate) struct SpanData {
  /// When the span was created (for duration calculation)
  pub created_at: Instant,
  /// Span name
  pub name: String,
  /// Target (module path)
  pub target: String,
  /// Log level
  pub level: tracing::Level,
  /// Source file (if available)
  pub file: Option<&'static str>,
  /// Source line (if available)
  pub line: Option<u32>,
  /// Fields captured at span creation (serialized as JSON)
  pub fields: serde_json::Value,
}
