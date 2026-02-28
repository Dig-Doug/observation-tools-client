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
    obs.metadata.contains_key("duration_s"),
    "Expected duration_s metadata"
  );
  assert!(
    obs.metadata.contains_key("duration_ns"),
    "Expected duration_ns metadata"
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

  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(obs.observation_type, ObservationType::LogEntry);
  assert_eq!(obs.log_level, LogLevel::Info);

  // Check payload is the message text
  let payload = obs.payload().as_str().expect("Expected text payload");
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
    .find(|o| o.observation_type == ObservationType::LogEntry)
    .expect("Expected event observation");
  let inner = observations
    .iter()
    .find(|o| o.name == "inner")
    .expect("Expected inner span");
  let outer = observations
    .iter()
    .find(|o| o.name == "outer")
    .expect("Expected outer span");

  // outer should have no parent (it's the root span)
  assert!(
    outer.parent_group_id.is_none(),
    "Outer span should not have a parent"
  );

  // inner's parent_group_id should reference outer's group ID
  let inner_parent = inner
    .parent_group_id
    .as_ref()
    .expect("Inner span should have a parent group");
  assert_eq!(
    inner_parent,
    &outer.group_ids[0],
    "Inner span's parent should be outer span"
  );

  // The event's parent_span_id should reference inner's group ID
  assert_eq!(
    event.parent_span_id.as_deref(),
    Some(inner.group_ids[0].as_str()),
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
      observe!("my_observation").serde(&"test data");
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

  assert_eq!(span_obs.observation_type, ObservationType::Group);

  // The observe!() observation's parent_span_id should match the span's group ID
  assert_eq!(
    my_obs.parent_span_id.as_deref(),
    Some(span_obs.group_ids[0].as_str()),
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

  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(
    obs.payload().as_str(),
    Some("user event"),
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
  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(obs.observation_type, ObservationType::Group);
  assert_eq!(obs.metadata.get("request_id"), Some(&"42".to_string()));
  assert_eq!(obs.metadata.get("user"), Some(&"alice".to_string()));
  assert_eq!(obs.metadata.get("enabled"), Some(&"true".to_string()));
  assert_eq!(obs.metadata.get("latency"), Some(&"1.5".to_string()));

  Ok(())
}
