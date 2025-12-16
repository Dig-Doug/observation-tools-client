mod common;

use common::TestServer;
use observation_tools_client::server_client::types::PayloadOrPointerResponse;

#[test_log::test(tokio::test)]
async fn test_api_without_auth_when_no_key_configured() -> anyhow::Result<()> {
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

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_api_with_valid_key() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  let execution = client
    .begin_execution("test-execution-with-observation")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  observation_tools_client::with_execution(execution, async {
    observation_tools_client::ObservationBuilder::new("test-observation")
      .label("test/label")
      .metadata("key1", "value1")
      .payload("test payload data")
      .build()
      .wait_for_upload()
      .await?;

    Ok::<_, anyhow::Error>(())
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "test-observation");
  assert_eq!(obs.payload.as_str(), Some("test payload data"));

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_api_without_auth_when_key_required() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let client = server.create_client()?;

  let result = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await;

  assert!(
    result.is_err(),
    "Request without auth should fail when API key is required"
  );

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_api_with_invalid_key() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let invalid_key = "obs_invalid_key_that_wont_work";
  let client = server.create_client_with_api_key(invalid_key)?;

  let result = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await;

  assert!(result.is_err(), "Request with invalid API key should fail");

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_api_with_malformed_key() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let malformed_key = "not-even-close-to-valid";
  let client = server.create_client_with_api_key(malformed_key)?;

  let result = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await;

  assert!(
    result.is_err(),
    "Request with malformed API key should fail"
  );

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_api_with_wrong_secret() -> anyhow::Result<()> {
  let server1 = TestServer::new_with_auth().await?;
  let api_key1 = server1.generate_api_key()?;

  let server2 = TestServer::new_with_auth().await?;
  let client = server2.create_client_with_api_key(&api_key1)?;

  let result = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await;

  assert!(
    result.is_err(),
    "API key from different server should fail validation"
  );

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_list_executions_with_auth() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  for i in 0..5 {
    let exec_name = format!("execution-{}", i);
    client
      .begin_execution(&exec_name)?
      .wait_for_upload()
      .await?;
  }

  let api_client = server.create_api_client_with_api_key(&api_key)?;
  let list_response = api_client.list_executions().send().await?;
  assert_eq!(list_response.executions.len(), 5);

  client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_blob_upload_with_auth() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  let execution = client
    .begin_execution("test-execution-with-large-payload")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let large_data = "x".repeat(70_000);
  let large_payload = serde_json::json!({
    "data": large_data,
    "size": large_data.len(),
  });

  let observation_id = observation_tools_client::with_execution(execution, async {
    observation_tools_client::observe!(
      name = "large-observation",
      label = "test/large-payload",
      payload = large_payload
    )
    .wait_for_upload()
    .await
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "large-observation");

  assert!(
    matches!(obs.payload, PayloadOrPointerResponse::Pointer { .. }),
    "Large payload data should be empty in metadata (stored as blob)"
  );

  let blob_url = format!(
    "{}/api/exe/{}/obs/{}/content",
    server.base_url(),
    execution_id,
    observation_id.id()
  );

  let response = reqwest::Client::new()
    .get(&blob_url)
    .header("Authorization", format!("Bearer {}", api_key))
    .send()
    .await?;

  assert!(
    response.status().is_success(),
    "Blob retrieval with auth should succeed"
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_multiple_valid_keys() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key1 = server.generate_api_key()?;
  let api_key2 = server.generate_api_key()?;

  let client1 = server.create_client_with_api_key(&api_key1)?;
  let client2 = server.create_client_with_api_key(&api_key2)?;

  let _execution1 = client1
    .begin_execution("execution-from-key1")?
    .wait_for_upload()
    .await?;

  let _execution2 = client2
    .begin_execution("execution-from-key2")?
    .wait_for_upload()
    .await?;

  client1.shutdown().await?;
  client2.shutdown().await?;

  let api_client1 = server.create_api_client_with_api_key(&api_key1)?;
  let list_response = api_client1.list_executions().send().await?;
  assert_eq!(list_response.executions.len(), 2);

  let api_client2 = server.create_api_client_with_api_key(&api_key2)?;
  let list_response = api_client2.list_executions().send().await?;
  assert_eq!(list_response.executions.len(), 2);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observation_without_auth_after_execution_with_auth() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let auth_client = server.create_client_with_api_key(&api_key)?;

  let execution = auth_client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await?;

  auth_client.shutdown().await?;

  let unauth_client = server.create_client()?;
  let result = observation_tools_client::with_execution(execution, async {
    observation_tools_client::observe!(name = "test-observation", payload = "test data")
      .wait_for_upload()
      .await
  })
  .await;

  assert!(
    result.is_err(),
    "Observation should fail when sent with unauthenticated client"
  );

  unauth_client.shutdown().await?;
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_list_executions_without_auth_succeeds() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await?;

  client.shutdown().await?;

  let api_client_no_auth = server.create_api_client()?;
  let result = api_client_no_auth.list_executions().send().await;

  assert!(
    result.is_ok(),
    "List executions should succeed without auth (read-only operation)"
  );

  let response = result?;
  assert_eq!(response.executions.len(), 1);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_get_execution_without_auth_succeeds() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  let execution = client
    .begin_execution("test-execution")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();
  client.shutdown().await?;

  let api_client_no_auth = server.create_api_client()?;
  let result = api_client_no_auth
    .get_execution()
    .id(&execution_id.to_string())
    .send()
    .await;

  assert!(
    result.is_ok(),
    "Get execution should succeed without auth (read-only operation)"
  );

  let response = result?;
  assert_eq!(response.execution.id.to_string(), execution_id.to_string());

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_blob_retrieval_without_auth_succeeds() -> anyhow::Result<()> {
  let server = TestServer::new_with_auth().await?;
  let api_key = server.generate_api_key()?;
  let client = server.create_client_with_api_key(&api_key)?;

  let execution = client
    .begin_execution("test-execution-with-large-payload")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let large_data = "x".repeat(70_000);
  let large_payload = serde_json::json!({
    "data": large_data,
  });

  let observation_id = observation_tools_client::with_execution(execution, async {
    observation_tools_client::observe!(name = "large-observation", payload = large_payload)
      .wait_for_upload()
      .await
  })
  .await?;

  client.shutdown().await?;

  let blob_url = format!(
    "{}/api/exe/{}/obs/{}/content",
    server.base_url(),
    execution_id,
    observation_id.id()
  );

  let response = reqwest::get(&blob_url).await?;
  assert!(
    response.status().is_success(),
    "Blob retrieval should succeed without auth (read-only operation)"
  );

  Ok(())
}
