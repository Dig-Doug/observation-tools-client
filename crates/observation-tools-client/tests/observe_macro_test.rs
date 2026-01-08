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
      observe!("simple_string").serde(&"hello world")
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

      observe!("serde_struct").serde(&MyStruct {
        message: "test message".to_string(),
        count: 42,
      })
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

      impl From<CustomStruct> for Payload {
        fn from(value: CustomStruct) -> Self {
          Payload::text(value.message)
        }
      }

      observe!("custom_payload").payload(CustomStruct {
        message: "custom message".to_string(),
      })
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

      impl From<CustomStruct> for Payload {
        fn from(value: CustomStruct) -> Self {
          Payload::text(format!("custom: {}", value.value))
        }
      }

      observe!("custom_new_syntax").payload(CustomStruct {
        value: "test".to_string(),
      })
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
      observe!(my_data).serde(&my_data)
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
async fn test_observe_with_label() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-structured", async {
      observe!("structured_observation")
        .label("test/category")
        .serde(&"test payload")
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
async fn test_observe_with_metadata() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-metadata", async {
      let duration_ms = 123;
      observe!("with_metadata")
        .metadata("request_type", "GET")
        .metadata("status_code", "200")
        .metadata("duration_ms", duration_ms.to_string())
        .serde(&"data")
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
async fn test_observe_const_name() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-expr-name", async {
      const OBSERVATION_NAME: &str = "const_name";
      observe!(OBSERVATION_NAME).serde(&"test data")
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
      observe!(&name).serde(&"test payload")
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
      observe!("request").label(label).serde(&"data")
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

#[test_log::test(tokio::test)]
async fn test_observe_debug_struct() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let (execution, observation) = server
    .with_execution("test-debug-struct", async {
      // This struct only implements Debug, not Serialize
      #[derive(Debug)]
      struct DebugOnlyStruct {
        #[allow(unused)]
        name: String,
        #[allow(unused)]
        value: i32,
      }

      let data = DebugOnlyStruct {
        name: "test".to_string(),
        value: 42,
      };

      observe!("debug_struct").debug(&data)
    })
    .await?;

  let observations = server.list_observations(&execution.id()).await?;

  assert_eq!(observations.len(), 1);
  let obs = &observations[0];
  assert_eq!(obs.name, "debug_struct");
  assert_eq!(obs.mime_type, "text/x-rust-debug");

  // The payload should be parsed to JSON with _type field
  let json = obs
    .payload
    .as_json()
    .expect("payload should be parsed as JSON");
  assert_eq!(
    json.get("_type"),
    Some(&serde_json::json!("DebugOnlyStruct"))
  );
  assert_eq!(json.get("name"), Some(&serde_json::json!("test")));
  assert_eq!(json.get("value"), Some(&serde_json::json!(42)));

  let response = reqwest::get(&observation.handle().url()).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}
