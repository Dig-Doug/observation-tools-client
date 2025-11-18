//! Basic usage example for observation-tools-client

use observation_tools_client::observe;
use observation_tools_client::ClientBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Initialize tracing with env filter
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .init();

  // Create a client
  let client = ClientBuilder::default()
    .base_url("http://localhost:3000")
    .build()?;

  // Begin an execution
  let execution = client.begin_execution("example_program")?.into_handle();
  println!("Started execution with ID {}", execution.id());

  // Register it globally so observe! macro can use it
  observation_tools_client::register_global_execution(execution)?;

  // Log some observations using the macro
  observe!("startup", "Application started")?;

  // Simulate some work with JSON data
  // For complex types, use the builder API with .payload()
  let data = serde_json::json!({
      "user_id": 123,
      "action": "login"
  });
  observation_tools_client::ObservationBuilder::new("user_event")
    .payload(data)
    .source(file!(), line!())
    .build()?;

  // With labels
  observe!(
    name = "api_call",
    label = "http/request",
    payload = "GET /api/users"
  )?;

  println!("Logged observations. Check the web UI at http://localhost:3000");

  // Graceful shutdown to flush pending observations
  client.shutdown().await?;

  // Give it a moment to finish uploads
  tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

  Ok(())
}
