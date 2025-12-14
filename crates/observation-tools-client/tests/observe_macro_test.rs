mod common;

use common::TestServer;
use observation_tools_client::observe;
use serde::Serialize;

#[test_log::test(tokio::test)]
async fn test_observe_simple_string_payload() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-simple-string")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let handle = observe!("simple_string", "hello world")
      .wait_for_upload()
      .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "simple_string");
  assert_eq!(
    obs.payload.as_json(),
    Some(&serde_json::to_value("hello world")?)
  );
  assert_eq!(obs.mime_type, "application/json");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_serde_struct() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-serde-struct")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    #[derive(Serialize)]
    struct MyStruct {
      message: String,
      count: i32,
    }

    let handle = observe!(
      "serde_struct",
      MyStruct {
        message: "test message".to_string(),
        count: 42,
      }
    )
    .wait_for_upload()
    .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "serde_struct");
  assert_eq!(
    obs.payload.as_json(),
    Some(&serde_json::from_str(
      r#"{"message":"test message","count":42}"#
    )?)
  );
  assert_eq!(obs.mime_type, "application/json");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_custom_payload() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-custom-payload")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    struct CustomStruct {
      message: String,
    }

    impl observation_tools_client::IntoCustomPayload for CustomStruct {
      fn to_payload(&self) -> observation_tools_client::Payload {
        observation_tools_client::Payload::text(self.message.clone())
      }
    }

    let handle = observe!(
      "custom_payload",
      CustomStruct {
        message: "custom message".to_string()
      },
      custom_serialization = true
    )
    .wait_for_upload()
    .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "custom_payload");
  assert_eq!(obs.payload.as_str(), Some("custom message"));
  assert_eq!(obs.mime_type, "text/plain");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_custom_with_new_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-custom-new-syntax")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    struct CustomStruct {
      value: String,
    }

    impl observation_tools_client::IntoCustomPayload for CustomStruct {
      fn to_payload(&self) -> observation_tools_client::Payload {
        observation_tools_client::Payload::text(format!("custom: {}", self.value))
      }
    }

    let handle = observe!(
      "custom_new_syntax",
      CustomStruct {
        value: "test".to_string()
      },
      custom = true
    )
    .wait_for_upload()
    .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "custom_new_syntax");
  assert_eq!(obs.payload.as_str(), Some("custom: test"));
  assert_eq!(obs.mime_type, "text/plain");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_variable_name_capture() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-var-capture")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let my_data = "captured variable name";
    let handle = observe!(my_data).wait_for_upload().await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  // The observation name should match the variable name
  assert_eq!(obs.name, "my_data");
  assert_eq!(obs.payload.as_str(), Some("\"captured variable name\""));

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_structured_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-structured")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let handle = observe!(
      name = "structured_observation",
      payload = "test payload",
      label = "test/category"
    )
    .wait_for_upload()
    .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "structured_observation");
  assert_eq!(obs.labels, vec!["test/category"]);
  assert_eq!(obs.payload.as_str(), Some("\"test payload\""));

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_metadata_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-metadata")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let duration_ms = 123;
    let handle = observe!(
      name = "with_metadata",
      payload = "data",
      metadata {
        request_type: "GET",
        status_code: "200",
        duration_ms: duration_ms.to_string()
      }
    )
    .wait_for_upload()
    .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "with_metadata");
  assert_eq!(obs.metadata.get("request_type"), Some(&"GET".to_string()));
  assert_eq!(obs.metadata.get("status_code"), Some(&"200".to_string()));
  assert_eq!(obs.metadata.get("duration_ms"), Some(&"123".to_string()));

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_expression_name() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-expr-name")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    const OBSERVATION_NAME: &str = "const_name";

    let handle = observe!(name = OBSERVATION_NAME, payload = "test data")
      .wait_for_upload()
      .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "const_name");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_dynamic_name() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-dynamic-name")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let prefix = "dynamic";
    let name = format!("{}_observation", prefix);

    let handle = observe!(&name, "test payload").wait_for_upload().await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "dynamic_observation");

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_dynamic_label() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let execution = client
    .begin_execution("test-dynamic-label")?
    .wait_for_upload()
    .await?;

  let execution_id = execution.id();

  let observation = observation_tools_client::with_execution(execution, async {
    let endpoint = "users";
    let label = format!("api/{}/create", endpoint);

    let handle = observe!(name = "request", payload = "data", label = label)
      .wait_for_upload()
      .await?;

    Ok::<_, anyhow::Error>(handle)
  })
  .await?;

  client.shutdown().await?;

  let observations = server.list_observations(&execution_id).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "request");
  assert_eq!(obs.labels, vec!["api/users/create"]);

  let response = reqwest::get(&observation.url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}
