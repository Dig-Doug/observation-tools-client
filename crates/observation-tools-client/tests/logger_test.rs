mod common;

use common::TestServer;
use observation_tools::observe;
use observation_tools::server_client::types::LogLevel;
use observation_tools::server_client::types::ObservationType;
use observation_tools::ObservationLogger;

#[tokio::test]
async fn test_logger() -> anyhow::Result<()> {
  ObservationLogger::init()?;

  let server = TestServer::new().await;
  let execution = server
    .with_execution("test-logger", async {
      log::info!("info-log");
      observe!("direct-log", "direct-log-payload");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 2);
  let obs = &observations[0];
  assert_eq!(obs.payload.as_str(), Some("info-log"));
  assert_eq!(obs.observation_type, ObservationType::LogEntry);
  assert_eq!(obs.log_level, LogLevel::Info);

  Ok(())
}
