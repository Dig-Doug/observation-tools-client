//! Integration tests for the Observation Tools Server API
//!
//! These tests start a local server instance and verify the API endpoints
//! work correctly using HTTP requests.

use observation_tools_shared::api::{CreateExecutionRequest, CreateExecutionResponse, GetExecutionResponse};
use observation_tools_shared::models::Execution;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start a test server on a random port
async fn start_test_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
  // Create a temporary directory for test data
  let data_dir = tempfile::tempdir().expect("Failed to create temp dir");

  // Bind to port 0 to get a random available port
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
    .await
    .expect("Failed to bind to random port");

  // Get the actual bound address
  let bind_addr = listener.local_addr().expect("Failed to get local address");

  let config = observation_tools_server::Config::new()
    .with_bind_addr(bind_addr)
    .with_data_dir(data_dir.path().to_path_buf());

  let server = observation_tools_server::Server::new(config);

  // Spawn the server in a background task
  let handle = tokio::spawn(async move {
    // Keep the tempdir alive for the duration of the server
    let _data_dir = data_dir;
    server.run(listener).await.expect("Server failed to run");
  });

  // Give the server a moment to start up
  sleep(Duration::from_millis(300)).await;

  (bind_addr, handle)
}

/// Helper to build the base URL for API requests
fn api_url(addr: &SocketAddr, path: &str) -> String {
  format!("http://{}/api{}", addr, path)
}

#[tokio::test]
async fn test_create_execution() {
  // Start the test server
  let (addr, _handle) = start_test_server().await;

  // Create a reqwest client
  let client = reqwest::Client::new();

  // Create an execution using the helper method
  let execution = Execution::new("test-execution");

  let request = CreateExecutionRequest { execution: execution.clone() };

  // Send the request
  let response = client
    .post(&api_url(&addr, "/exe"))
    .json(&request)
    .send()
    .await
    .expect("Failed to send request");

  // Check that the response is successful
  assert!(
    response.status().is_success(),
    "Expected successful response, got: {} - {}",
    response.status(),
    response.text().await.unwrap_or_default()
  );

  // Parse the response (note: CreateExecutionResponse is empty)
  let _create_response: CreateExecutionResponse = response
    .json()
    .await
    .expect("Failed to parse response");

  // Verify we can retrieve the execution
  let get_response = client
    .get(&api_url(&addr, &format!("/exe/{}", execution.id)))
    .send()
    .await
    .expect("Failed to get execution");

  assert!(get_response.status().is_success());

  let get_data: GetExecutionResponse = get_response
    .json()
    .await
    .expect("Failed to parse get response");

  // Verify the execution details
  assert_eq!(get_data.execution.id, execution.id);
  assert_eq!(get_data.execution.name, execution.name);
}
