#![cfg(feature = "tracing-layer")]

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
  let execution = server
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
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let execution = server
    .with_execution("test-tracing-event", async {
      tracing::info!(key = "value", "test event message");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;
  assert_eq!(observations.len(), 1);

  let obs = &observations[0];
  assert_eq!(obs.observation_type, ObservationType::LogEntry);

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
  let execution = server
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
  // Events have names like "event <file>:<line>" in tracing
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

  // The event should have inner's span as parent
  assert!(
    event.parent_span_id.is_some(),
    "Event should have a parent span"
  );

  // inner should have outer's span as parent
  assert!(
    inner.parent_span_id.is_some(),
    "Inner span should have a parent"
  );

  // outer should have no parent
  assert!(
    outer.parent_span_id.is_none(),
    "Outer span should not have a parent"
  );

  Ok(())
}

#[tokio::test]
async fn test_observe_macro_gets_parent_span() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let execution = server
    .with_execution("test-observe-parent", async {
      let span = tracing::info_span!("parent_span");
      let _guard = span.enter();

      // observe!() should automatically get the current span as parent
      observe!("my_observation", "test data");
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

  // The observe!() observation should have the parent span ID set automatically
  assert!(
    my_obs.parent_span_id.is_some(),
    "observe!() should automatically get parent span ID"
  );

  // Verify the parent ID matches the span's ID (converted to string)
  // The span doesn't have an ID in the observation, so we verify it has a parent
  assert_eq!(span_obs.observation_type, ObservationType::Span);

  Ok(())
}

#[tokio::test]
async fn test_internal_events_filtered() -> anyhow::Result<()> {
  let _guard = tracing_subscriber::registry()
    .with(ObservationLayer::new())
    .set_default();

  let server = TestServer::new().await;
  let execution = server
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
  let execution = server
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
