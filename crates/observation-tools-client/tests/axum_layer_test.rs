//! Integration tests for Axum layers
//!
//! Tests the ExecutionLayer and RequestObserverLayer middleware.

#![cfg(feature = "axum")]

mod common;

use anyhow::anyhow;
use axum::response::sse::Event;
use axum::response::sse::Sse;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use common::TestServer;
use futures::stream;
use futures::StreamExt;
use http::header::HeaderName;
use observation_tools::axum::ExecutionLayer;
use observation_tools::axum::RequestObserverConfig;
use observation_tools::axum::RequestObserverLayer;
use observation_tools::observe;
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;

#[test_log::test(tokio::test)]
async fn test_execution_layer_creates_context() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;
  let app = Router::new()
    .route(
      "/hello",
      get(|| async {
        observe!("handler_observation").serde(&"Hello from handler").build();
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
  assert_eq!(observations.len(), 4);
  assert_eq!(observations[0].name, "http/request/headers");
  assert_eq!(observations[1].name, "http/request/body");
  assert_eq!(observations[2].name, "http/response/headers");
  assert_eq!(observations[3].name, "http/response/body");
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
  let payload = observations[0]
    .payload
    .as_json()
    .ok_or(anyhow!("Not json"))?;
  let headers = payload.as_object().expect("headers should be object");
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
        observe!("error_observation").serde(&"An error occurred").build();
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
  assert_eq!(observations.len(), 5);
  assert_eq!(
    observations[3].log_level,
    observation_tools::server_client::types::LogLevel::Error,
    "5xx responses should have Error log level"
  );

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_captures_request_and_response_body() -> anyhow::Result<()> {
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

  assert_eq!(observations.len(), 4);

  // Check request observation has body with base64 data and content-type
  let request_payload = observations[1]
    .payload
    .as_json()
    .ok_or(anyhow!("Not json"))?;
  assert_eq!(request_payload, &request_body);

  // Check response observation has body with base64 data and content-type
  let response_payload = observations[3]
    .payload
    .as_json()
    .ok_or(anyhow!("Not json"))?;
  assert_eq!(response_payload["message"], "echo response");
  assert_eq!(response_payload["received"]["name"], "test");

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_request_observer_handles_text_body() -> anyhow::Result<()> {
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

  println!("{:#?}", observations);
  let response_payload = observations[3]
    .payload
    .as_str()
    .ok_or(anyhow!("Not string"))?;
  assert_eq!(response_payload, "Hello, World!");

  Ok(())
}

#[test_log::test(tokio::test)]
async fn test_execution_layer_filter_skips_execution() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  let app = Router::new()
    .route(
      "/health",
      get(|| async {
        observe!("health_observation").serde(&"Health check").build();
        "OK"
      }),
    )
    .route(
      "/api",
      get(|| async {
        observe!("api_observation").serde(&"API call").build();
        "API Response"
      }),
    )
    .layer(
      ExecutionLayer::new(client.clone()).with_filter(|req| {
        // Skip health check endpoints
        !req.uri().path().starts_with("/health")
      }),
    );

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  // Call health endpoint - should be filtered out
  let response = reqwest::get(format!("http://{}/health", addr)).await?;
  assert_eq!(response.status(), 200);
  assert_eq!(response.text().await?, "OK");

  // Call API endpoint - should create execution
  let response = reqwest::get(format!("http://{}/api", addr)).await?;
  assert_eq!(response.status(), 200);
  assert_eq!(response.text().await?, "API Response");

  client.shutdown().await?;

  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;

  // Only one execution should be created (for /api)
  assert_eq!(executions.executions.len(), 1);
  assert!(executions.executions[0].name.contains("/api"));

  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;
  assert_eq!(observations.len(), 1);
  assert_eq!(observations[0].name, "api_observation");

  Ok(())
}

/// Test that Server-Sent Events (SSE) are streamed in real-time through the observer layer.
/// This verifies that the layer doesn't buffer the entire response before returning it.
#[test_log::test(tokio::test)]
async fn test_streaming_sse_events_are_received_incrementally() -> anyhow::Result<()> {
  let server = TestServer::new().await;
  let client = server.create_client()?;

  // Create an SSE endpoint that sends events with delays between them
  let app = Router::new()
    .route(
      "/sse",
      get(|| async {
        let stream = stream::iter(vec![
          Ok::<_, Infallible>(Event::default().data("event1")),
          Ok(Event::default().data("event2")),
          Ok(Event::default().data("event3")),
        ])
        .then(|event| async move {
          // Small delay between events to make streaming observable
          tokio::time::sleep(Duration::from_millis(50)).await;
          event
        });
        Sse::new(stream)
      }),
    )
    .layer(RequestObserverLayer::new())
    .layer(ExecutionLayer::new(client.clone()));

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  tokio::spawn(async move {
    axum::serve(listener, app).await.expect("Server failed");
  });

  // Create a channel to track when each event is received
  let (tx, mut rx) = mpsc::channel::<(usize, Instant)>(10);
  let start = Instant::now();

  // Make the SSE request
  let response = reqwest::get(format!("http://{}/sse", addr)).await?;
  assert_eq!(response.status(), 200);

  // Process the SSE stream
  let mut event_count = 0;
  let mut stream = response.bytes_stream();

  while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    let text = String::from_utf8_lossy(&chunk);
    // Count events by looking for "data:" lines
    for line in text.lines() {
      if line.starts_with("data:") {
        event_count += 1;
        tx.send((event_count, Instant::now())).await?;
      }
    }
  }

  drop(tx);

  // Collect all the timing data
  let mut timings = Vec::new();
  while let Some((idx, time)) = rx.recv().await {
    timings.push((idx, time.duration_since(start)));
  }

  // Verify we received all 3 events
  assert_eq!(
    timings.len(),
    3,
    "Expected 3 SSE events, got {}",
    timings.len()
  );

  // Verify the events were received incrementally (not all at once)
  // If streaming works, there should be measurable time gaps between events
  // The delays are 50ms, so we expect at least some gap between first and last event
  if timings.len() >= 2 {
    let first_event_time = timings[0].1;
    let last_event_time = timings[timings.len() - 1].1;
    let time_spread = last_event_time - first_event_time;

    // With 50ms delays between events, we expect at least 80ms spread across 3 events
    // (accounting for some timing variance)
    assert!(
      time_spread >= Duration::from_millis(80),
      "Events should be spread over time (streaming), but received with only {:?} spread. \
             This suggests responses are being buffered instead of streamed.",
      time_spread
    );
  }

  client.shutdown().await?;

  // Verify the observations were still captured
  let api_client = server.create_api_client()?;
  let executions = api_client.list_executions().send().await?;
  assert_eq!(executions.executions.len(), 1);

  let observations = server
    .list_observations(&executions.executions[0].id)
    .await?;

  // Should have 4 observations: request headers, request body, response headers, response body
  assert_eq!(observations.len(), 4);
  assert_eq!(observations[0].name, "http/request/headers");
  assert_eq!(observations[1].name, "http/request/body");
  assert_eq!(observations[2].name, "http/response/headers");
  assert_eq!(observations[3].name, "http/response/body");

  // Verify the response body was captured
  // SSE uses text/event-stream which is stored as InlineBinary
  use observation_tools::server_client::types::PayloadOrPointerResponse;
  let response_body = match &observations[3].payload {
    PayloadOrPointerResponse::Text(t) => t.clone(),
    PayloadOrPointerResponse::InlineBinary(data) => {
      let bytes: Vec<u8> = data.iter().map(|&x| x as u8).collect();
      String::from_utf8_lossy(&bytes).to_string()
    }
    other => anyhow::bail!("Unexpected payload type: {:?}", other),
  };
  assert!(
    response_body.contains("event1"),
    "Response body should contain event1, got: {}",
    response_body
  );
  assert!(
    response_body.contains("event2"),
    "Response body should contain event2, got: {}",
    response_body
  );
  assert!(
    response_body.contains("event3"),
    "Response body should contain event3, got: {}",
    response_body
  );

  Ok(())
}
