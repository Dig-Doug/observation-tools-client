//! Integration tests for Axum layers
//!
//! Tests the ExecutionLayer and RequestObserverLayer middleware.

#![cfg(feature = "axum")]

mod common;

use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use common::PayloadExt;
use common::TestServer;
use http::header::HeaderName;
use observation_tools_client::axum::ExecutionLayer;
use observation_tools_client::axum::RequestObserverConfig;
use observation_tools_client::axum::RequestObserverLayer;
use observation_tools_client::observe;
use serde_json::json;

#[test_log::test(tokio::test)]
async fn test_execution_layer_creates_context() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let app = Router::new()
    .route(
      "/hello",
      get(|| async {
        observe!("handler_observation", "Hello from handler");
        "Hello, World!"
      }),
    )
    .layer(ExecutionLayer::new(client.clone()));
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let response = reqwest::get(format!("http://{}/hello", addr)).await?;
  assert_eq!(response.status(), 200);
  assert_eq!(response.text().await?, "Hello, World!");
  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  assert_eq!(executions.executions.len(), 1);
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;
  assert_eq!(observations.len(), 1);
  assert_eq!(observations[0].name, "handler_observation");

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_captures_request_response() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let app = Router::new()
    .route("/test", get(|| async { "OK" }))
    .layer(RequestObserverLayer::new())
    .layer(ExecutionLayer::new(client.clone()));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let response = reqwest::get(format!("http://{}/test", addr)).await?;
  assert_eq!(response.status(), 200);
  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;
  assert_eq!(observations.len(), 2);
  assert_eq!(observations[0].name, "http/request");
  assert_eq!(observations[1].name, "http/response");
  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_without_execution_context() -> anyhow::Result<()> {
  // RequestObserverLayer without ExecutionLayer should not panic,
  // it should just skip observation
  let app = Router::new()
    .route("/test", get(|| async { "OK" }))
    .layer(RequestObserverLayer::new());
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let response = reqwest::get(format!("http://{}/test", addr)).await?;
  assert_eq!(response.status(), 200);

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_config_excludes_headers() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let config =
    RequestObserverConfig::new().exclude_header(HeaderName::from_static("x-custom-secret"));
  let app = Router::new()
    .route("/test", get(|| async { "OK" }))
    .layer(RequestObserverLayer::with_config(config))
    .layer(ExecutionLayer::new(client.clone()));
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });
  let http_client = reqwest::Client::new();
  let response = http_client
    .get(format!("http://{}/test", addr))
    .header("x-custom-secret", "my-secret-value")
    .header("x-safe-header", "safe-value")
    .bearer_auth("my-secret-value")
    .send()
    .await?;
  assert_eq!(response.status(), 200);
  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;
  let payload: serde_json::Value =
    serde_json::from_slice(&observations[0].payload.data_as_bytes())?;
  let headers = payload["headers"]
    .as_object()
    .expect("headers should be object");
  assert!(
    !headers.contains_key("x-custom-secret"),
    "x-custom-secret should be excluded"
  );
  assert!(
    headers.contains_key("x-safe-header"),
    "x-safe-header should be included"
  );
  assert!(
    !headers.contains_key("authorization"),
    "authorization should be excluded by default"
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_error_response_has_error_log_level() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let app = Router::new()
    .route(
      "/error",
      get(|| async {
        observe!("error_observation", "An error occurred");
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Error")
      }),
    )
    .layer(RequestObserverLayer::new())
    .layer(ExecutionLayer::new(client.clone()));
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let response = reqwest::get(format!("http://{}/error", addr)).await?;
  assert_eq!(response.status(), 500);
  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;
  assert_eq!(observations.len(), 3);
  assert_eq!(
    observations[2].log_level,
    observation_tools_client::server_client::types::LogLevel::Error,
    "5xx responses should have Error log level"
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_captures_request_and_response_body() -> anyhow::Result<()> {
  use base64::Engine;

  let server = TestServer::new().await;
  let client = server.create_client()?;

  let app = Router::new()
    .route(
      "/echo",
      post(|Json(body): Json<serde_json::Value>| async move {
        Json(json!({
            "received": body,
            "message": "echo response"
        }))
      }),
    )
    .layer(RequestObserverLayer::new())
    .layer(ExecutionLayer::new(client.clone()));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let http_client = reqwest::Client::new();
  let request_body = json!({
      "name": "test",
      "value": 42
  });
  let response = http_client
    .post(format!("http://{}/echo", addr))
    .json(&request_body)
    .send()
    .await?;
  assert_eq!(response.status(), 200);

  let response_json: serde_json::Value = response.json().await?;
  assert_eq!(response_json["received"]["name"], "test");
  assert_eq!(response_json["message"], "echo response");

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;

  assert_eq!(observations.len(), 2);

  // Check request observation has body with base64 data and content-type
  let request_payload: serde_json::Value =
    serde_json::from_slice(&observations[0].payload.data_as_bytes())?;
  assert!(
    request_payload.get("body").is_some(),
    "request should have body"
  );
  let request_body_obj = &request_payload["body"];
  assert!(
    request_body_obj["content_type"]
      .as_str()
      .is_some_and(|ct| ct.starts_with("application/json")),
    "request body should have application/json content-type"
  );
  let request_data = request_body_obj["data"]
    .as_str()
    .expect("data should be string");
  let decoded_request = base64::engine::general_purpose::STANDARD.decode(request_data)?;
  let decoded_request_json: serde_json::Value = serde_json::from_slice(&decoded_request)?;
  assert_eq!(decoded_request_json["name"], "test");
  assert_eq!(decoded_request_json["value"], 42);

  // Check response observation has body with base64 data and content-type
  let response_payload: serde_json::Value =
    serde_json::from_slice(&observations[1].payload.data_as_bytes())?;
  assert!(
    response_payload.get("body").is_some(),
    "response should have body"
  );
  let response_body_obj = &response_payload["body"];
  assert!(
    response_body_obj["content_type"]
      .as_str()
      .is_some_and(|ct| ct.starts_with("application/json")),
    "response body should have application/json content-type"
  );
  let response_data = response_body_obj["data"]
    .as_str()
    .expect("data should be string");
  let decoded_response = base64::engine::general_purpose::STANDARD.decode(response_data)?;
  let decoded_response_json: serde_json::Value = serde_json::from_slice(&decoded_response)?;
  assert_eq!(decoded_response_json["message"], "echo response");
  assert_eq!(decoded_response_json["received"]["name"], "test");

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_handles_text_body() -> anyhow::Result<()> {
  use base64::Engine;

  let server = TestServer::new().await;
  let client = server.create_client()?;

  let app = Router::new()
    .route("/text", get(|| async { "Hello, World!" }))
    .layer(RequestObserverLayer::new())
    .layer(ExecutionLayer::new(client.clone()));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  let response = reqwest::get(format!("http://{}/text", addr)).await?;
  assert_eq!(response.status(), 200);
  assert_eq!(response.text().await?, "Hello, World!");

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;

  // Check response observation has body with base64 data and content-type
  let response_payload: serde_json::Value =
    serde_json::from_slice(&observations[1].payload.data_as_bytes())?;
  let response_body_obj = &response_payload["body"];
  assert!(
    response_body_obj["content_type"]
      .as_str()
      .is_some_and(|ct| ct.starts_with("text/plain")),
    "response body should have text/plain content-type"
  );
  let response_data = response_body_obj["data"]
    .as_str()
    .expect("data should be string");
  let decoded_response = base64::engine::general_purpose::STANDARD.decode(response_data)?;
  let decoded_text = String::from_utf8(decoded_response)?;
  assert_eq!(decoded_text, "Hello, World!");

  Ok(())
}
