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
  let (execution, _) = server
    .with_execution("test-logger", async {
      log::info!("info-log");
      observe!("direct-log").serde(&"direct-log-payload");
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 2);
  let obs = server.get_observation(&execution.id(), &observations[0].id).await?;
  assert_eq!(obs.payload().as_str(), Some("info-log"));
  assert_eq!(obs.observation_type, ObservationType::LogEntry);
  assert_eq!(obs.log_level, LogLevel::Info);

  Ok(())
}
