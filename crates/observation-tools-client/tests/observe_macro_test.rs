mod common;

use common::TestServer;
use observation_tools::observe;
use observation_tools_shared::Payload;
use serde::Serialize;

#[test_log::test(tokio::test)]
async fn test_observe_simple_string_payload() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-simple-string", async {
      observe!("simple_string", "hello world")
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "simple_string");
  assert_eq!(
    obs.payload.as_json(),
    Some(&serde_json::to_value("hello world")?)
  );
  assert_eq!(obs.mime_type, "application/json");

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_serde_struct() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-serde-struct", async {
      #[derive(Serialize)]
      struct MyStruct {
        message: String,
        count: i32,
      }

      observe!(
        "serde_struct",
        MyStruct {
          message: "test message".to_string(),
          count: 42,
        }
      )
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

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

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_custom_payload() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-custom-payload", async {
      struct CustomStruct {
        message: String,
      }

      impl From<&CustomStruct> for Payload {
        fn from(value: &CustomStruct) -> Self {
          Payload::text(value.message.clone())
        }
      }

      observe!(
        "custom_payload",
        CustomStruct {
          message: "custom message".to_string()
        },
        custom_serialization = true
      )
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "custom_payload");
  assert_eq!(obs.payload.as_str(), Some("custom message"));
  assert_eq!(obs.mime_type, "text/plain");

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_custom_with_new_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-custom-new-syntax", async {
      struct CustomStruct {
        value: String,
      }

      impl From<&CustomStruct> for Payload {
        fn from(value: &CustomStruct) -> Self {
          Payload::text(format!("custom: {}", value.value))
        }
      }

      observe!(
        "custom_new_syntax",
        CustomStruct {
          value: "test".to_string()
        },
        custom = true
      )
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "custom_new_syntax");
  assert_eq!(obs.payload.as_str(), Some("custom: test"));
  assert_eq!(obs.mime_type, "text/plain");

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_variable_name_capture() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-var-capture", async {
      let my_data = "captured variable name";
      observe!(my_data)
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  // The observation name should match the variable name
  assert_eq!(obs.name, "my_data");
  assert_eq!(
    obs.payload.as_json(),
    Some(&serde_json::to_value("captured variable name")?)
  );

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_structured_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-structured", async {
      observe!(
        name = "structured_observation",
        payload = "test payload",
        label = "test/category"
      )
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "structured_observation");
  assert_eq!(obs.labels, vec!["test/category"]);
  assert_eq!(
    obs.payload.as_json(),
    Some(&serde_json::to_value("test payload")?)
  );

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_metadata_syntax() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-metadata", async {
      let duration_ms = 123;
      observe!(
        name = "with_metadata",
        payload = "data",
        metadata {
          request_type: "GET",
          status_code: "200",
          duration_ms: duration_ms.to_string()
        }
      )
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "with_metadata");
  assert_eq!(obs.metadata.get("request_type"), Some(&"GET".to_string()));
  assert_eq!(obs.metadata.get("status_code"), Some(&"200".to_string()));
  assert_eq!(obs.metadata.get("duration_ms"), Some(&"123".to_string()));

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_expression_name() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-expr-name", async {
      const OBSERVATION_NAME: &str = "const_name";
      observe!(name = OBSERVATION_NAME, payload = "test data")
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "const_name");

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_dynamic_name() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-dynamic-name", async {
      let prefix = "dynamic";
      let name = format!("{}_observation", prefix);
      observe!(&name, "test payload")
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "dynamic_observation");

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_observe_dynamic_label() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-dynamic-label", async {
      let endpoint = "users";
      let label = format!("api/{}/create", endpoint);
      observe!(name = "request", payload = "data", label = label)
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "request");
  assert_eq!(obs.labels, vec!["api/users/create"]);

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}
