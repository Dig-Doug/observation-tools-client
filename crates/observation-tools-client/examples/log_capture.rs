//! Example demonstrating log capture functionality
//!
//! This example shows how to use the ObservationLogger to automatically
//! capture log messages as observations.

use anyhow::Result;
use observation_tools_client::{ClientBuilder, ObservationLogger};

#[tokio::main]
async fn main() -> Result<()> {
  // Initialize the observation logger
  // This will capture all log messages and convert them to observations
  ObservationLogger::init_with_level(log::Level::Debug)?;

  // Create a client
  let client = ClientBuilder::new()
    .base_url("http://localhost:3000")
    .build()?;

  // Begin an execution
  let execution = client
    .begin_execution("log-capture-example")?
    .wait_for_upload()
    .await?;

  println!("Execution URL: {}", execution.url());

  // Register the execution as the global context
  // This allows the logger to automatically attach logs to this execution
  observation_tools_client::register_global_execution(execution.clone())?;

  // Now all log messages will be captured as observations
  log::info!("Application started");
  log::debug!("Debug information");
  log::warn!("This is a warning");

  // You can still use the observe! macro for structured data
  observation_tools_client::observe!("user_action", "click_button")?;

  // Simulate some work with logging
  perform_work().await?;

  log::info!("Application finished");

  // Give some time for observations to be uploaded
  tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

  // Shutdown the client
  let _ = client.shutdown().await;

  println!("\nCheck the execution URL to see the captured logs!");

  Ok(())
}

async fn perform_work() -> Result<()> {
  log::info!("Starting work");

  for i in 1..=5 {
    log::debug!("Processing item {}", i);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
  }

  log::info!("Work completed");
  Ok(())
}
