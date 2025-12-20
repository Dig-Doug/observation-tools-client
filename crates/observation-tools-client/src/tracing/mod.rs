//! Tracing integration for observation-tools.

mod layer;
mod span_data;

pub use layer::ObservationLayer;

pub fn current_span_id() -> Option<String> {
  let current = tracing::Span::current();
  current.id().map(|id| id.into_u64().to_string())
}

pub fn test_only_generate_internal_events() {
  log::info!("log entry");
  tracing::info!("tracing event");
  let s = tracing::info_span!("tracing span");
  let _g = s.enter();
}
