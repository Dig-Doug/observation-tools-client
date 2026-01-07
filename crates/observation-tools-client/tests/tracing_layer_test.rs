#![cfg(feature = "tracing")]

mod common;

use common::TestServer;
use observation_tools::observe;
use observation_tools::server_client::types::ObservationType;
use observation_tools::tracing::ObservationLayer;
use tracing_subscriber::prelude::*;

#[tokio::test]
async fn test_span_captured_on_close() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-tracing-span", async {
      let span = tracing::info_span!("test_span", key = "value");
      let _guard = span.enter();
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;
  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name.as_str(), "test_span");
  assert!(
    obs.metadata.contains_key("duration_ms"),
    "Expected duration_ms metadata"
  );

  Ok(())
}

#[tokio::test]
async fn test_event_captured() -> anyhow::Result<()> {
  use observation_tools::server_client::types::LogLevel;

  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-tracing-event", async {
      tracing::info!(key = "value", "test event message");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;
  assert_eq!(observations.len(), 1);

  let obs = &observations[0];
  assert_eq!(obs.observation_type, ObservationType::LogEntry);
  assert_eq!(obs.log_level, LogLevel::Info);

  // Check payload is the message text
  let payload = obs.payload.as_str().expect("Expected text payload");
  assert_eq!(payload, "test event message");

  // Check event fields are in metadata
  assert_eq!(obs.metadata.get("key"), Some(&"value".to_string()));

  Ok(())
}

#[tokio::test]
async fn test_no_observations_without_execution_context() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-no-context")?
    .wait_for_upload()
    .await?;

  tracing::info!("this should be ignored");
  let span = tracing::info_span!("ignored_span");
  drop(span.enter());
  client.shutdown().await?;

  let observations = server.list_observations(&execution.id()).await?;
  assert!(
    observations.is_empty(),
    "Expected no observations, got {:?}",
    observations.len()
  );

  Ok(())
}

#[tokio::test]
async fn test_parent_span_attribution() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-parent-span", async {
      let outer = tracing::info_span!("outer");
      let _outer_guard = outer.enter();

      let inner = tracing::info_span!("inner");
      let _inner_guard = inner.enter();

      tracing::info!("event inside inner span");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  // Find our specific observations
  let event = observations
    .iter()
    .find(|o| {
      o.observation_type == ObservationType::LogEntry
        && o.labels.iter().any(|l| l.starts_with("tracing/events"))
    })
    .expect("Expected event observation");
  let inner = observations
    .iter()
    .find(|o| o.name == "inner")
    .expect("Expected inner span");
  let outer = observations
    .iter()
    .find(|o| o.name == "outer")
    .expect("Expected outer span");

  // Get span IDs from metadata
  let outer_span_id = outer
    .metadata
    .get("span_id")
    .expect("Outer span should have span_id metadata");
  let inner_span_id = inner
    .metadata
    .get("span_id")
    .expect("Inner span should have span_id metadata");

  // outer should have no parent (it's the root span)
  assert!(
    outer.parent_span_id.is_none(),
    "Outer span should not have a parent"
  );

  // inner's parent_span_id should match outer's span_id
  assert_eq!(
    inner.parent_span_id.as_ref(),
    Some(outer_span_id),
    "Inner span's parent should be outer span"
  );

  // The event's parent_span_id should match inner's span_id
  assert_eq!(
    event.parent_span_id.as_ref(),
    Some(inner_span_id),
    "Event's parent should be inner span"
  );

  Ok(())
}

#[tokio::test]
async fn test_observe_macro_gets_parent_span() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-observe-parent", async {
      let span = tracing::info_span!("parent_span");
      let _guard = span.enter();

      // observe!() should automatically get the current span as parent
      observe!("my_observation").serde(&"test data").build();
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  // Find the observe!() observation (it has Payload type, not Span)
  let my_obs = observations
    .iter()
    .find(|o| o.name == "my_observation")
    .expect("Expected my_observation");

  // Find the span observation
  let span_obs = observations
    .iter()
    .find(|o| o.name == "parent_span")
    .expect("Expected parent_span observation");

  assert_eq!(span_obs.observation_type, ObservationType::Span);

  // Get the span's tracing ID from metadata
  let span_id = span_obs
    .metadata
    .get("span_id")
    .expect("parent_span should have span_id metadata");

  // The observe!() observation's parent_span_id should match the span's span_id
  assert_eq!(
    my_obs.parent_span_id.as_ref(),
    Some(span_id),
    "observe!() should have parent_span as its parent"
  );

  Ok(())
}

#[tokio::test]
async fn test_internal_events_filtered() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-internal-filter", async {
      // Generate internal log/tracing events from observation_tools crate
      // These should NOT be captured as observations
      observation_tools::tracing::test_only_generate_internal_events();

      // This user event SHOULD be captured
      tracing::info!(target: "user_app", "user event");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  // Should only have the user event, not the internal ones
  // The internal events are: log entry, tracing event, tracing span
  assert_eq!(
    observations.len(),
    1,
    "Expected only 1 observation (user event), but got {}. Observations: {:?}",
    observations.len(),
    observations.iter().map(|o| &o.name).collect::<Vec<_>>()
  );

  let obs = &observations[0];
  assert!(
    obs.labels.iter().any(|l| l.contains("user_app")),
    "The captured observation should be the user event"
  );

  Ok(())
}

#[tokio::test]
async fn test_span_with_multiple_fields() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-span-fields", async {
      let span = tracing::info_span!(
        "test_span",
        request_id = 42,
        user = "alice",
        enabled = true,
        latency = 1.5
      );
      let _guard = span.enter();
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;
  let obs = &observations[0];
  assert_eq!(obs.observation_type, ObservationType::Span);
  let fields = obs.payload.as_json().expect("Expected JSON payload");
  assert_eq!(fields["request_id"], 42);
  assert_eq!(fields["user"], "alice");
  assert_eq!(fields["enabled"], true);
  assert_eq!(fields["latency"], 1.5);

  Ok(())
}
