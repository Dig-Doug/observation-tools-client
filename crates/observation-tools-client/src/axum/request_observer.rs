//! Request/Response observer middleware for Axum
//!
//! Automatically observes incoming requests and outgoing responses.

use crate::context;
use crate::observation::ObservationBuilder;
use axum::extract::Request;
use axum::response::Response;
use http::header::HeaderMap;
use http::header::HeaderName;
use http::header::AUTHORIZATION;
use http::header::COOKIE;
use http::header::SET_COOKIE;
use observation_tools_shared::LogLevel;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tower::Layer;
use tower::Service;

/// Configuration for RequestObserverLayer
#[derive(Clone)]
pub struct RequestObserverConfig {
  /// Headers to exclude from observation
  pub excluded_headers: Vec<HeaderName>,
}

impl Default for RequestObserverConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl RequestObserverConfig {
  /// Create a new configuration with defaults
  ///
  /// By default, excludes sensitive headers: authorization, cookie, set-cookie
  pub fn new() -> Self {
    Self {
      excluded_headers: vec![AUTHORIZATION, COOKIE, SET_COOKIE],
    }
  }

  /// Add a header to exclude from observation
  pub fn exclude_header(mut self, header: HeaderName) -> Self {
    self.excluded_headers.push(header);
    self
  }
}

/// Layer that observes HTTP requests and responses
///
/// This layer depends on `ExecutionLayer` being applied first (as an outer
/// layer) to provide the execution context.
#[derive(Clone)]
pub struct RequestObserverLayer {
  config: RequestObserverConfig,
}

impl RequestObserverLayer {
  pub fn new() -> Self {
    Self {
      config: RequestObserverConfig::new(),
    }
  }

  pub fn with_config(config: RequestObserverConfig) -> Self {
    Self { config }
  }
}

impl Default for RequestObserverLayer {
  fn default() -> Self {
    Self::new()
  }
}

impl<S> Layer<S> for RequestObserverLayer {
  type Service = RequestObserverService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    RequestObserverService {
      inner,
      config: self.config.clone(),
    }
  }
}

/// Service that observes requests and responses
#[derive(Clone)]
pub struct RequestObserverService<S> {
  inner: S,
  config: RequestObserverConfig,
}

impl<S> Service<Request> for RequestObserverService<S>
where
  S: Service<Request, Response = Response> + Clone + Send + 'static,
  S::Future: Send,
{
  type Response = S::Response;
  type Error = S::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request) -> Self::Future {
    let config = self.config.clone();
    let mut inner = self.inner.clone();

    Box::pin(async move {
      if !context::get_current_execution().is_some() {
        tracing::debug!(
          "RequestObserverLayer: No execution context available, skipping observation"
        );
        return inner.call(req).await;
      }

      let mut request_builder = ObservationBuilder::new("http/request");
      request_builder
        .label("http/request")
        .metadata("method", req.method().to_string())
        .metadata("uri", req.uri().to_string());
      let request_payload = json!({
          "headers": filter_headers(req.headers(), &config.excluded_headers),
      });
      let _ = request_builder.payload(&request_payload).build();

      let response = inner.call(req).await?;

      let status = response.status();
      let log_level = match status.as_u16() {
        200..=299 => LogLevel::Info,
        400..=499 => LogLevel::Warning,
        500..=599 => LogLevel::Error,
        _ => LogLevel::Info,
      };
      let mut response_builder = ObservationBuilder::new("http/response");
      response_builder
        .label("http/response")
        .metadata("status", &status.as_u16().to_string())
        .log_level(log_level);
      let response_payload = json!({
          "headers": filter_headers(response.headers(), &config.excluded_headers),
      });
      let _ = response_builder.payload(&response_payload).build();

      Ok(response)
    })
  }
}

/// Filter headers, removing excluded ones and converting to a JSON-friendly
/// format
fn filter_headers(
  headers: &HeaderMap,
  excluded: &[HeaderName],
) -> serde_json::Map<String, serde_json::Value> {
  let mut map = serde_json::Map::new();
  for (name, value) in headers.iter() {
    if !excluded.contains(name) {
      if let Ok(v) = value.to_str() {
        map.insert(
          name.as_str().to_string(),
          serde_json::Value::String(v.to_string()),
        );
      }
    }
  }
  map
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_filter_headers_excludes_sensitive() {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("authorization", "Bearer secret".parse().unwrap());
    headers.insert("x-custom", "value".parse().unwrap());

    let excluded = vec![AUTHORIZATION];
    let filtered = filter_headers(&headers, &excluded);

    assert!(filtered.contains_key("content-type"));
    assert!(filtered.contains_key("x-custom"));
    assert!(!filtered.contains_key("authorization"));
  }

  #[test]
  fn test_default_config_excludes_sensitive_headers() {
    let config = RequestObserverConfig::new();
    assert!(config.excluded_headers.contains(&AUTHORIZATION));
    assert!(config.excluded_headers.contains(&COOKIE));
    assert!(config.excluded_headers.contains(&SET_COOKIE));
  }

  #[test]
  fn test_config_exclude_header() {
    let config = RequestObserverConfig::new().exclude_header(HeaderName::from_static("x-api-key"));
    assert!(config
      .excluded_headers
      .contains(&HeaderName::from_static("x-api-key")));
  }
}
