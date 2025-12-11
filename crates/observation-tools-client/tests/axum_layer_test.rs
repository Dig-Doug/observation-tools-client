//! Integration tests for Axum layers
//!
//! Tests the ExecutionLayer and RequestObserverLayer middleware.

#![cfg(feature = "axum")]

mod common;

use axum::routing::get;
use axum::Router;
use common::TestServer;
use http::header::HeaderName;
use observation_tools_client::axum::ExecutionLayer;
use observation_tools_client::axum::RequestObserverConfig;
use observation_tools_client::axum::RequestObserverLayer;
use observation_tools_client::observe;

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
  let payload: serde_json::Value = serde_json::from_str(&observations[0].payload.data)?;
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
