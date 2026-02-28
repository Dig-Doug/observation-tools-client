use super::span_data::SpanData;
use crate::context;
use crate::group::GroupBuilder;
use crate::group::GroupHandle;
use crate::observation::ObservationBuilder;
use observation_tools_shared::GroupId;
use observation_tools_shared::LogLevel;
use observation_tools_shared::ObservationType;
use observation_tools_shared::Payload;
use std::time::Instant;
use tracing::span::Attributes;
use tracing::Event;
use tracing::Id;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// A tracing layer that captures spans and events as observations.
///
/// This layer integrates with the Observation Tools client to capture:
/// - Spans (on close only, with duration)
/// - Events (immediately when they occur)
#[derive(Clone, Default)]
pub struct ObservationLayer;

impl ObservationLayer {
  /// Create a new ObservationLayer
  pub fn new() -> Self {
    Self
  }
}

impl<S> Layer<S> for ObservationLayer
where
  S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
  fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
    // Self-filtering: skip spans from observation_tools crate
    let metadata = attrs.metadata();
    if metadata.target().starts_with("observation_tools") {
      return;
    }

    // Also check module_path for spans
    if let Some(module_path) = metadata.module_path() {
      if module_path.starts_with("observation_tools") {
        return;
      }
    }

    // Store span data in extensions for later use
    let Some(span) = ctx.span(id) else {
      return;
    };

    // Capture fields as JSON
    let mut visitor = FieldVisitor::new();
    attrs.record(&mut visitor);

    let data = SpanData {
      created_at: Instant::now(),
      name: metadata.name().to_string(),
      target: metadata.target().to_string(),
      level: *metadata.level(),
      file: metadata.file(),
      line: metadata.line(),
      fields: visitor.into_json(),
    };

    span.extensions_mut().insert(data);
  }

  fn on_close(&self, id: Id, ctx: Context<'_, S>) {
    // Skip if no execution context
    let Some(execution) = context::get_current_execution() else {
      return;
    };

    let Some(span) = ctx.span(&id) else {
      return;
    };

    let extensions = span.extensions();
    let Some(data) = extensions.get::<SpanData>() else {
      // Span was filtered out during on_new_span
      return;
    };

    let builder = GroupBuilder::from_span(
      &data.name,
      tracing_level_to_log_level(data.level),
      span
        .parent()
        .map(|parent| GroupId::from(parent.id().into_u64().to_string())),
    )
    .id(id.into_u64().to_string())
    .metadata("target", &data.target);

    let duration = data.created_at.elapsed();
    let mut builder = builder
      .metadata("duration_s", duration.as_secs().to_string())
      .metadata("duration_ns", duration.subsec_nanos().to_string());

    if let serde_json::Value::Object(fields) = &data.fields {
      for (key, value) in fields {
        let value_str = match value {
          serde_json::Value::String(s) => s.clone(),
          other => other.to_string(),
        };
        builder = builder.metadata(key, value_str);
      }
    }

    if let (Some(file), Some(line)) = (data.file, data.line) {
      builder = builder.source(file, line);
    }

    let _ = builder.build_with_execution(&execution);
  }

  fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
    // Self-filtering: skip events from observation_tools crate
    let metadata = event.metadata();
    if let Some(module_path) = metadata.module_path() {
      if module_path.starts_with("observation_tools") {
        return;
      }
    }

    let Some(execution) = context::get_current_execution() else {
      return;
    };

    let mut visitor = FieldVisitor::new();
    event.record(&mut visitor);

    // Self-filtering: Check log.module_path field for events coming through
    // tracing-log
    if let Some(serde_json::Value::String(module_path)) = visitor.fields.get("log.module_path") {
      if module_path.starts_with("observation_tools") {
        return;
      }
    }

    // Extract the message field as the payload, use remaining fields as metadata
    let message = visitor
      .fields
      .remove("message")
      .and_then(|v| match v {
        serde_json::Value::String(s) => Some(s),
        other => Some(other.to_string()),
      })
      .unwrap_or_default();

    // Get current span from tracing's span stack for parent attribution
    let current_span_id = ctx.current_span().id().map(|id| id.into_u64());
    let parent_span_id = current_span_id.map(|id| id.to_string());

    // Build and send observation
    let mut builder = ObservationBuilder::new(metadata.name())
      .observation_type(ObservationType::LogEntry)
      .log_level(tracing_level_to_log_level(*metadata.level()));

    // Add group reference from current span
    if let Some(span_id) = current_span_id {
      let group_handle = GroupHandle::from_id(GroupId::from(span_id.to_string()), &execution);
      builder = builder.group(&group_handle);
    }

    let builder = if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
      builder.source(file, line)
    } else {
      builder
    };

    let builder = if let Some(parent_id) = parent_span_id {
      builder.parent_span_id(parent_id)
    } else {
      builder
    };

    // Add remaining event fields as metadata
    let builder = visitor.fields.into_iter().fold(builder, |b, (key, value)| {
      let value_str = match value {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
      };
      b.metadata(key, value_str)
    });

    let _ = builder.payload(Payload::text(message));
  }
}

/// Convert tracing::Level to observation_tools_shared::LogLevel
fn tracing_level_to_log_level(level: tracing::Level) -> LogLevel {
  match level {
    tracing::Level::TRACE => LogLevel::Trace,
    tracing::Level::DEBUG => LogLevel::Debug,
    tracing::Level::INFO => LogLevel::Info,
    tracing::Level::WARN => LogLevel::Warning,
    tracing::Level::ERROR => LogLevel::Error,
  }
}

/// Visitor for capturing tracing fields as serde_json::Value
struct FieldVisitor {
  fields: serde_json::Map<String, serde_json::Value>,
}

impl FieldVisitor {
  fn new() -> Self {
    Self {
      fields: serde_json::Map::new(),
    }
  }

  fn into_json(self) -> serde_json::Value {
    serde_json::Value::Object(self.fields)
  }
}

impl tracing::field::Visit for FieldVisitor {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    self.fields.insert(
      field.name().to_string(),
      serde_json::Value::String(format!("{:?}", value)),
    );
  }

  fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
    self
      .fields
      .insert(field.name().to_string(), serde_json::json!(value));
  }

  fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
    self
      .fields
      .insert(field.name().to_string(), serde_json::json!(value));
  }

  fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
    self
      .fields
      .insert(field.name().to_string(), serde_json::json!(value));
  }

  fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
    self.fields.insert(
      field.name().to_string(),
      serde_json::json!(value.to_string()),
    );
  }

  fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
    self.fields.insert(
      field.name().to_string(),
      serde_json::json!(value.to_string()),
    );
  }

  fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
    self
      .fields
      .insert(field.name().to_string(), serde_json::json!(value));
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    self.fields.insert(
      field.name().to_string(),
      serde_json::Value::String(value.to_string()),
    );
  }

  fn record_error(
    &mut self,
    field: &tracing::field::Field,
    value: &(dyn std::error::Error + 'static),
  ) {
    self.fields.insert(
      field.name().to_string(),
      serde_json::Value::String(value.to_string()),
    );
  }
}
