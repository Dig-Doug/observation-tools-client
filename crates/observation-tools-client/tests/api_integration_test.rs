//! Integration tests for the Observation Tools Server API
//!
//! These tests start a local server instance and verify the API endpoints
//! work correctly using both the client library and the OpenAPI client.

mod common;

use anyhow::anyhow;
use common::TestServer;
use observation_tools::observe;
use observation_tools::server_client::types::PayloadOrPointerResponse;
use observation_tools::ObservationBuilder;
use observation_tools_shared::Payload;
use serde_json::json;
use std::collections::HashSet;
use tracing::debug;

#[test_log::test(tokio::test)]
async fn test_create_execution_with_client() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution_handle = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await?;

  let execution_id = execution_handle.id();
  let api_client = server.create_api_client()?;
  let get_response = api_client
    .get_execution()
    .id(&execution_id.to_string())
    .send()
    .await?;
  assert_eq!(
    get_response.execution.id.to_string(),
    execution_id.to_string()
  );
  assert_eq!(get_response.execution.name, "test-execution");
  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_create_observation_with_metadata() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, _) = server
    .with_execution("test-execution-with-observation", async {
      observation_tools::ObservationBuilder::new("test-observation")
        .metadata("key1", "value1")
        .metadata("key2", "value2")
        .payload("test payload data");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);

  // Get the full observation with inline payload data
  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(obs.name, "test-observation");
  assert_eq!(obs.execution_id.to_string(), execution.id().to_string());
  assert_eq!(obs.metadata.get("key1"), Some(&"value1".to_string()));
  assert_eq!(obs.metadata.get("key2"), Some(&"value2".to_string()));
  assert_eq!(obs.payloads[0].mime_type, "text/plain");
  assert_eq!(obs.payload().as_str(), Some("test payload data"));

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_create_many_observations() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let execution = client
    .begin_execution("test-execution-with-many-observations")?
    .wait_for_upload()
    .await?;

  // Create a payload that is exactly 1 byte smaller than the blob threshold
  // String payloads are now stored without JSON quotes, so the payload size
  // is exactly the string length
  let payload_data = "x".repeat(observation_tools::BLOB_THRESHOLD_BYTES - 1);
  let expected_names = observation_tools::with_execution(execution.clone(), async {
    let mut expected_names = HashSet::new();
    // Create BATCH_SIZE observations to test batching behavior
    for i in 0..observation_tools::BATCH_SIZE {
      let obs_name = format!("observation-{}", i);
      observe!(&obs_name).payload(Payload::text(&payload_data));
      expected_names.insert(obs_name);
    }

    Ok::<_, anyhow::Error>(expected_names)
  })
  .await?;
  client.shutdown().await?;

  let observations = server.list_observations(&execution.id()).await?;
  assert_eq!(observations.len(), expected_names.len());

  // List returns metadata only (payloads as Pointers)
  // Verify payload sizes are correct
  for obs in &observations {
    assert_eq!(
      obs.payloads[0].size as u64,
      observation_tools::BLOB_THRESHOLD_BYTES as u64 - 1,
      "Observation {} payload size should be exactly 1 byte under threshold",
      obs.name
    );
    // When listing, all payloads come back as Pointers
    assert!(
      matches!(obs.payload(), PayloadOrPointerResponse::Pointer { .. }),
      "Observation {} should have Pointer payload in list response",
      obs.name
    );
  }

  let obs_names: HashSet<String> = observations.iter().map(|o| o.name.clone()).collect();
  assert_eq!(obs_names.difference(&expected_names).count(), 0);
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_list_executions() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let mut expected_names = HashSet::new();
  for i in 0..57 {
    let exec_name = format!("execution-{}", i);
    client
      .begin_execution(&exec_name)?
      .wait_for_upload()
      .await?;
    expected_names.insert(exec_name);
  }

  let api_client = server.create_api_client()?;
  let list_response = api_client.list_executions().send().await?;
  assert_eq!(list_response.executions.len(), expected_names.len());
  let exec_names: HashSet<String> = list_response
    .executions
    .iter()
    .map(|e| e.name.clone())
    .collect();
  assert_eq!(exec_names.difference(&expected_names).count(), 0);

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_concurrent_executions() -> anyhow::Result<()> {
  const TASK_1_NAME: &str = "concurrent/task1";
  const TASK_2_NAME: &str = "concurrent/task2";
  const NUM_OBSERVATIONS: usize = 3;
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution1 = client.begin_execution("task-1")?.wait_for_upload().await?;
  let execution2 = client.begin_execution("task-2")?.wait_for_upload().await?;
  // Use channels to ensure that the tasks alternate
  let (task1_sender, mut task1_receiver) = tokio::sync::mpsc::unbounded_channel();
  let (task2_sender, mut task2_receiver) = tokio::sync::mpsc::unbounded_channel();
  tokio::try_join!(
    observation_tools::with_execution(execution1.clone(), async {
      for _ in 0..NUM_OBSERVATIONS {
        debug!("Task 1 sending observation");
        observation_tools::observe!(TASK_1_NAME)
          .serde(&"data from task 1")
          .wait_for_upload()
          .await?;
        let _ = task1_sender.send(());
        debug!("Task 1 waiting for task 2");
        let Some(_) = task2_receiver.recv().await else {
          return Err(anyhow!("Did not receive signal from task 2"));
        };
      }
      // Close the sender to signal completion. Without the drop, debug builds hang.
      drop(task1_sender);
      Ok::<_, anyhow::Error>(())
    }),
    observation_tools::with_execution(execution2.clone(), async {
      while let Some(_) = task1_receiver.recv().await {
        debug!("Task 2 sending observation");
        observation_tools::observe!(TASK_2_NAME)
          .serde(&"data from task 2")
          .wait_for_upload()
          .await?;
        debug!("Task 2 waiting for task 1");
        let _ = task2_sender.send(());
      }
      Ok::<_, anyhow::Error>(())
    })
  )?;

  client.shutdown().await?;

  let observations1 = server.list_observations(&execution1.id()).await?;
  let count1 = observations1
    .iter()
    .filter(|o| o.name == TASK_1_NAME)
    .count();
  assert_eq!(count1, NUM_OBSERVATIONS);

  let observations2 = server.list_observations(&execution2.id()).await?;
  let count2 = observations2
    .iter()
    .filter(|o| o.name == TASK_2_NAME)
    .count();
  assert_eq!(count2, NUM_OBSERVATIONS);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_with_observations_spawned_task() -> anyhow::Result<()> {
  use observation_tools::WithObservations;

  const SPAWNED_OBS_NAME: &str = "spawned-task-observation";
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-with-observations")?
    .wait_for_upload()
    .await?;

  // Use with_execution to set up the context, then spawn a task with
  // with_observations
  observation_tools::with_execution(execution.clone(), async {
    // Spawn a task that uses with_observations to inherit the execution context
    let handle = tokio::spawn(
      async move {
        // This observation should be associated with the parent execution
        observe!(SPAWNED_OBS_NAME)
          .serde(&"data from spawned task")
          .wait_for_upload()
          .await
      }
      .with_observations(),
    );

    handle.await.expect("Spawned task failed")
  })
  .await?;

  client.shutdown().await?;

  // Verify the observation from the spawned task was associated with the correct
  // execution
  let observations = server.list_observations(&execution.id()).await?;
  assert_eq!(observations.len(), 1);
  assert_eq!(observations[0].name, SPAWNED_OBS_NAME);
  assert_eq!(
    observations[0].execution_id.to_string(),
    execution.id().to_string()
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_with_observations_no_context() -> anyhow::Result<()> {
  use observation_tools::WithObservations;

  // Test that with_observations works even when there's no execution context
  // (the future should just run normally without panicking)
  let result = async { 42 }.with_observations().await;
  assert_eq!(result, 42);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_large_payload_blob_upload() -> anyhow::Result<()> {
  use futures::TryStreamExt;

  let server = TestServer::new().await;

  // Create a large payload (>64KB threshold)
  let large_data = "x".repeat(70_000);
  let large_payload = serde_json::json!({
    "data": large_data,
    "size": large_data.len(),
    "description": "This is a large payload that should be stored as a blob"
  });

  // Calculate the expected size (serialized JSON)
  let expected_size = serde_json::to_string(&large_payload)?.len();

  let (execution, _) = server
    .with_execution("test-execution-with-large-payload", async {
      observe!("large-observation")
        .serde(&large_payload);
    })
    .await?;

  // Verify the observation metadata was stored
  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "large-observation");

  // The payload.data should be empty because it was uploaded as a blob
  assert!(
    matches!(obs.payload(), PayloadOrPointerResponse::Pointer { .. }),
    "Large payload data should be stored as a blob pointer"
  );

  // But the size should still be recorded
  assert_eq!(
    obs.payloads[0].size as u64, expected_size as u64,
    "Payload size should be recorded correctly"
  );

  // Verify the blob can be retrieved via the OpenAPI client
  let api_client = server.create_api_client()?;
  let payload_id = &obs.payloads[0].id;
  let blob_response = api_client
    .get_observation_blob()
    .execution_id(&execution.id().to_string())
    .observation_id(&obs.id.to_string())
    .payload_id(&payload_id.to_string())
    .send()
    .await?;

  let blob_bytes: bytes::Bytes = blob_response
    .into_inner_stream()
    .try_collect::<Vec<_>>()
    .await?
    .into_iter()
    .flatten()
    .collect();

  let retrieved_payload: serde_json::Value = serde_json::from_slice(&blob_bytes)?;

  // Verify the retrieved content matches what we uploaded
  assert_eq!(
    retrieved_payload["data"].as_str().unwrap().len(),
    70_000,
    "Retrieved blob should have the correct data length"
  );
  assert_eq!(
    retrieved_payload["size"].as_u64().unwrap(),
    70_000,
    "Retrieved blob should have the correct size field"
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_named_payloads() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let execution = client
    .begin_execution("test-named-payloads")?
    .wait_for_upload()
    .await?;

  observation_tools::with_execution(execution.clone(), async {
    // Create an observation with a named payload, then add more payloads via the handle
    let handle = ObservationBuilder::new("multi-payload-obs")
      .metadata("kind", "multi")
      .named_serde("headers", &json!({"content-type": "application/json"}));

    handle.serde("body", &json!({"message": "hello"}));
    handle.payload(
      "raw-part",
      Payload::text("some raw text"),
    );
  })
  .await;

  client.shutdown().await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1, "Should have exactly one observation");

  // Get full observation with inline payload data
  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(obs.name, "multi-payload-obs");
  assert_eq!(obs.metadata.get("kind"), Some(&"multi".to_string()));

  // The observation should have 3 named payloads (headers, body, raw-part)
  assert_eq!(
    obs.payloads.len(), 3,
    "Should have 3 payloads (headers + body + raw-part), got {}",
    obs.payloads.len()
  );

  // Find the "headers" payload by name
  let headers_payload = obs
    .payloads
    .iter()
    .find(|p| p.name == "headers")
    .expect("Should have a 'headers' payload");
  let payload_json: serde_json::Value = match &headers_payload.data {
    PayloadOrPointerResponse::Json(v) => v.clone(),
    other => anyhow::bail!("Expected JSON payload for 'headers', got {:?}", other),
  };
  assert_eq!(
    payload_json,
    json!({"content-type": "application/json"}),
    "Headers payload should contain the headers data"
  );

  Ok(())
}
