mod common;

use common::TestServer;
use observation_tools_client::observe;
use observation_tools_client::server_client::types::LogLevel;
use observation_tools_client::server_client::types::ObservationType;
use observation_tools_client::ObservationLogger;

#[tokio::test]
async fn test_logger() -> anyhow::Result<()> {
  ObservationLogger::init()?;

  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-logger")?
    .wait_for_upload()
    .await?;
  let execution_id = execution.id();
  observation_tools_client::with_execution(execution, async {
    log::info!("info-log");
    observe!("direct-log", "direct-log-payload")?
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

  assert_eq!(list_response.observations.len(), 2);
  let obs = &list_response.observations[0];
  assert_eq!(obs.payload.data, "info-log");
  assert_eq!(obs.observation_type, ObservationType::LogEntry);
  assert_eq!(obs.log_level, LogLevel::Info);

  Ok(())
}
