//! Integration tests for the Observation Tools Server API
//!
//! These tests start a local server instance and verify the API endpoints
//! work correctly using both the client library and the OpenAPI client.

use anyhow::anyhow;
use observation_tools_client::observe;
use observation_tools_client::Client;
use observation_tools_client::ClientBuilder;
use observation_tools_shared::ExecutionId;
use serde::Serialize;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

/// Test server wrapper that provides convenient client creation
struct TestServer {
  addr: SocketAddr,
  _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
  /// Start a new test server on a random port
  async fn new() -> Self {
    let data_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
      .await
      .expect("Failed to bind to random port");

    let addr = listener.local_addr().expect("Failed to get local address");

    let config = observation_tools_server::Config::new()
      .with_bind_addr(addr)
      .with_data_dir(data_dir.path().to_path_buf());

    let server = observation_tools_server::Server::new(config);

    let handle = tokio::spawn(async move {
      // Keep the tempdir alive for the duration of the server
      let _data_dir = data_dir;
      server.run(listener).await.expect("Server failed to run");
    });

    // Give the server a moment to start up
    sleep(Duration::from_millis(300)).await;

    Self {
      addr,
      _handle: handle,
    }
  }

  /// Create an observation tools client connected to this test server
  fn create_client(&self) -> anyhow::Result<Client> {
    let base_url = format!("http://{}", self.addr);
    Ok(ClientBuilder::new().base_url(&base_url).build()?)
  }

  /// Create an OpenAPI client connected to this test server
  fn create_api_client(&self) -> anyhow::Result<observation_tools_client::server_client::Client> {
    let base_url = format!("http://{}", self.addr);
    observation_tools_client::server_client::create_client(&base_url)
  }
}

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
async fn test_observe_macro() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-observe_macro")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  observation_tools_client::with_execution(execution, async {
    // Simple string payload - uses default Serialize behavior
    observe!("no_args", "simple payload")?
      .wait_for_upload()
      .await?;

    // Struct with only Serialize - uses default JSON serialization
    #[derive(Serialize)]
    struct MySerdeStruct {
      message: String,
    }
    observe!(
      "serde_struct",
      MySerdeStruct {
        message: "hello".to_string()
      }
    )?
    .wait_for_upload()
    .await?;

    // Struct with custom IntoPayload - requires custom_serialization = true
    struct MyCustomPayloadStruct {
      message: String,
    }

    impl observation_tools_client::IntoCustomPayload for MyCustomPayloadStruct {
      fn to_payload(&self) -> observation_tools_client::Payload {
        observation_tools_client::Payload::text(self.message.clone())
      }
    }
    observe!(
      "custom_payload_struct",
      MyCustomPayloadStruct {
        message: "custom payload".to_string()
      },
      custom_serialization = true
    )?;

    // Struct with both Serialize AND IntoPayload - uses custom_serialization = true
    // to prefer IntoPayload
    #[derive(Serialize)]
    struct MyCustomPayloadWithSerdeStruct {
      message: String,
    }

    impl observation_tools_client::IntoCustomPayload for MyCustomPayloadWithSerdeStruct {
      fn to_payload(&self) -> observation_tools_client::Payload {
        observation_tools_client::Payload::text(self.message.clone())
      }
    }
    observe!(
      "custom_payload_with_serde_struct",
      MyCustomPayloadWithSerdeStruct {
        message: "custom payload with serde".to_string()
      },
      custom_serialization = true
    )?;

    Ok::<_, anyhow::Error>(())
  })
  .await?;

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let list_response = api_client
    .list_observations()
    .execution_id(&execution_id.to_string())
    .send()
    .await?;

  // Should have 4 observations
  assert_eq!(list_response.observations.len(), 4);

  // Find observations by name and verify their payloads
  let obs_map: std::collections::HashMap<_, _> = list_response
    .observations
    .iter()
    .map(|obs| (obs.name.as_str(), obs))
    .collect();

  // 1. Simple string payload - serialized as JSON (String implements Serialize)
  let no_args = obs_map.get("no_args").expect("no_args observation");
  assert_eq!(no_args.payload.data, "\"simple payload\""); // JSON string includes quotes
  assert_eq!(no_args.payload.mime_type, "application/json");

  // 2. Struct with Serialize - serialized to JSON
  let serde_struct = obs_map
    .get("serde_struct")
    .expect("serde_struct observation");
  assert_eq!(serde_struct.payload.data, r#"{"message":"hello"}"#);
  assert_eq!(serde_struct.payload.mime_type, "application/json");

  // 3. Struct with custom IntoPayload - uses custom serialization (text/plain)
  let custom = obs_map
    .get("custom_payload_struct")
    .expect("custom_payload_struct observation");
  assert_eq!(custom.payload.data, "custom payload");
  assert_eq!(custom.payload.mime_type, "text/plain");

  // 4. Struct with both Serialize and IntoPayload - uses custom IntoPayload due
  //    to custom_serialization = true
  let custom_with_serde = obs_map
    .get("custom_payload_with_serde_struct")
    .expect("custom_payload_with_serde_struct observation");
  assert_eq!(custom_with_serde.payload.data, "custom payload with serde");
  assert_eq!(custom_with_serde.payload.mime_type, "text/plain");

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_create_observation_with_metadata() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-execution-with-observation")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  observation_tools_client::with_execution(execution, async {
    observation_tools_client::ObservationBuilder::new("test-observation")
      .payload("test payload data")
      .label("test/label1")
      .label("test/label2")
      .metadata("key1", "value1")
      .metadata("key2", "value2")
      .build()?
      .wait_for_upload()
      .await?;

    Ok::<_, anyhow::Error>(())
  })
  .await?;

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let list_response = api_client
    .list_observations()
    .execution_id(&execution_id.to_string())
    .send()
    .await?;

  assert_eq!(list_response.observations.len(), 1);

  let obs = &list_response.observations[0];
  assert_eq!(obs.name, "test-observation");
  assert_eq!(obs.execution_id.to_string(), execution_id.to_string());
  assert!(obs.labels.contains(&"test/label1".to_string()));
  assert!(obs.labels.contains(&"test/label2".to_string()));
  assert_eq!(obs.metadata.get("key1"), Some(&"value1".to_string()));
  assert_eq!(obs.metadata.get("key2"), Some(&"value2".to_string()));
  assert_eq!(obs.payload.mime_type, "text/plain");
  assert_eq!(obs.payload.data, "test payload data");

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
  let payload_data = "x".repeat(observation_tools_client::BLOB_THRESHOLD_BYTES - 1);
  let expected_names = observation_tools_client::with_execution(execution.clone(), async {
    let mut expected_names = HashSet::new();
    // Create BATCH_SIZE observations to test batching behavior
    for i in 0..observation_tools_client::BATCH_SIZE {
      let obs_name = format!("observation-{}", i);
      observe!(&obs_name, payload_data);
      expected_names.insert(obs_name);
    }

    Ok::<_, anyhow::Error>(expected_names)
  })
  .await?;
  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let list_response = api_client
    .list_observations()
    .execution_id(&execution.id().to_string())
    .send()
    .await?;
  assert_eq!(list_response.observations.len(), expected_names.len());

  // Verify all payloads are stored inline (not as blobs) since they're under the
  // threshold
  for obs in &list_response.observations {
    assert!(
      !obs.payload.data.is_empty(),
      "Observation {} should have inline payload data (not stored as blob)",
      obs.name
    );
    assert_eq!(
      obs.payload.size,
      observation_tools_client::BLOB_THRESHOLD_BYTES as u64 - 1,
      "Observation {} payload size should be exactly 1 byte under threshold",
      obs.name
    );
  }

  let obs_names: HashSet<String> = list_response
    .observations
    .iter()
    .map(|o| o.name.clone())
    .collect();
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
    observation_tools_client::with_execution(execution1.clone(), async {
      for _ in 0..NUM_OBSERVATIONS {
        debug!("Task 1 sending observation");
        observation_tools_client::observe!(
          name = TASK_1_NAME,
          label = "concurrent/task1",
          payload = "data from task 1"
        )?
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
    observation_tools_client::with_execution(execution2.clone(), async {
      while let Some(_) = task1_receiver.recv().await {
        debug!("Task 2 sending observation");
        observation_tools_client::observe!(
          name = TASK_2_NAME,
          label = "concurrent/task2",
          payload = "data from task 2"
        )?
        .wait_for_upload()
        .await?;
        debug!("Task 2 waiting for task 1");
        let _ = task2_sender.send(());
      }
      Ok::<_, anyhow::Error>(())
    })
  )?;

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let count_observations_with_name =
    async |execution_id: ExecutionId, name: &str| -> anyhow::Result<usize> {
      let response = api_client
        .list_observations()
        .execution_id(&execution_id.to_string())
        .send()
        .await?;
      Ok(
        response
          .observations
          .iter()
          .filter(|o| o.name == name)
          .count(),
      )
    };

  assert_eq!(
    count_observations_with_name(execution1.id(), TASK_1_NAME).await?,
    NUM_OBSERVATIONS,
  );
  assert_eq!(
    count_observations_with_name(execution2.id(), TASK_2_NAME).await?,
    NUM_OBSERVATIONS
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_large_payload_blob_upload() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-execution-with-large-payload")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  // Create a large payload (>64KB threshold)
  let large_data = "x".repeat(70_000);
  let large_payload = serde_json::json!({
    "data": large_data,
    "size": large_data.len(),
    "description": "This is a large payload that should be stored as a blob"
  });

  // Calculate the expected size (serialized JSON)
  let expected_size = serde_json::to_string(&large_payload)?.len();

  let observation_id = observation_tools_client::with_execution(execution, async {
    observe!(
      name = "large-observation",
      label = "test/large-payload",
      payload = large_payload
    )?
    .wait_for_upload()
    .await
  })
  .await?;

  client.shutdown().await?;

  // Verify the observation metadata was stored
  let api_client = server.create_api_client()?;
  let list_response = api_client
    .list_observations()
    .execution_id(&execution_id.to_string())
    .send()
    .await?;

  assert_eq!(list_response.observations.len(), 1);
  let obs = &list_response.observations[0];
  assert_eq!(obs.name, "large-observation");
  assert_eq!(obs.id.to_string(), observation_id.to_string());

  // The payload.data should be empty because it was uploaded as a blob
  assert_eq!(
    obs.payload.data, "",
    "Large payload data should be empty in metadata (stored as blob)"
  );

  // But the size should still be recorded
  assert_eq!(
    obs.payload.size, expected_size as u64,
    "Payload size should be recorded correctly"
  );

  // Verify the blob can be retrieved via the content endpoint
  let blob_url = format!(
    "http://{}/api/exe/{}/obs/{}/content",
    server.addr, execution_id, observation_id
  );

  let response = reqwest::get(&blob_url).await?;
  assert!(
    response.status().is_success(),
    "Blob retrieval should succeed"
  );

  let content = response.text().await?;
  let retrieved_payload: serde_json::Value = serde_json::from_str(&content)?;

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
