use crate::context;
use crate::group::GroupBuilder;
use crate::observation::ObservationBuilder;
use crate::observation_handle::ObservationPayloadHandle;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use bytes::Bytes;
use bytes::BytesMut;
use http::header::HeaderMap;
use http::header::HeaderName;
use http::header::AUTHORIZATION;
use http::header::CONTENT_TYPE;
use http::header::COOKIE;
use http::header::SET_COOKIE;
use http_body::Frame;
use http_body_util::BodyExt;
use observation_tools_shared::LogLevel;
use observation_tools_shared::Payload;
use pin_project_lite::pin_project;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use tower::Layer;
use tower::Service;

/// State shared between the streaming body and the observation emitter.
/// This is used to collect data as it streams and add the body payload when
/// complete. The body payload is added in the Drop implementation via the
/// ObservationPayloadHandle.
struct StreamingObserverState {
  buffer: BytesMut,
  content_type: String,
  payload_handle: ObservationPayloadHandle,
}

impl StreamingObserverState {
  fn new(content_type: String, payload_handle: ObservationPayloadHandle) -> Self {
    Self {
      buffer: BytesMut::new(),
      content_type,
      payload_handle,
    }
  }

  fn append(&mut self, data: &Bytes) {
    self.buffer.extend_from_slice(data);
  }
}

impl Drop for StreamingObserverState {
  fn drop(&mut self) {
    let bytes = self.buffer.clone().freeze();
    tracing::debug!(
      "StreamingObserverBody: adding body payload with {} bytes on drop",
      bytes.len()
    );
    let payload = Payload {
      data: bytes.to_vec(),
      mime_type: self.content_type.clone(),
      size: bytes.len(),
    };

    self.payload_handle.raw_payload("body", payload);
  }
}

pin_project! {
  /// A body wrapper that streams data through while collecting it for observation.
  /// When the stream completes, it adds the body as a named payload to the
  /// existing response observation.
  pub struct StreamingObserverBody {
    #[pin]
    inner: Body,
    state: Arc<Mutex<StreamingObserverState>>,
  }
}

impl StreamingObserverBody {
  fn new(inner: Body, content_type: String, payload_handle: ObservationPayloadHandle) -> Self {
    Self {
      inner,
      state: Arc::new(Mutex::new(StreamingObserverState::new(
        content_type,
        payload_handle,
      ))),
    }
  }
}

impl http_body::Body for StreamingObserverBody {
  type Data = Bytes;
  type Error = axum::Error;

  fn poll_frame(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
    let this = self.project();

    match this.inner.poll_frame(cx) {
      Poll::Ready(Some(Ok(frame))) => {
        // If this is a data frame, capture the data
        if let Some(data) = frame.data_ref() {
          if let Ok(mut state) = this.state.lock() {
            state.append(data);
          }
        }
        Poll::Ready(Some(Ok(frame)))
      }
      Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }

  fn is_end_stream(&self) -> bool {
    self.inner.is_end_stream()
  }

  fn size_hint(&self) -> http_body::SizeHint {
    self.inner.size_hint()
  }
}

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
  pub fn new() -> Self {
    Self {
      excluded_headers: vec![AUTHORIZATION, COOKIE, SET_COOKIE],
    }
  }

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
      let execution = match context::get_current_execution() {
        Some(exec) => exec,
        None => {
          tracing::debug!(
            "RequestObserverLayer: No execution context available, skipping observation"
          );
          return inner.call(req).await;
        }
      };

      let (parts, body) = req.into_parts();

      // Create a single group for the HTTP exchange
      let http_group = GroupBuilder::new("http_request")
        .metadata("method", parts.method.to_string())
        .metadata("uri", parts.uri.to_string())
        .build_with_execution(&execution)
        .into_handle();

      // Collect request body
      let request_body_bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_else(|_| Bytes::new());

      // Single request observation with named payloads: "headers" + "body"
      let headers_json = json!(filter_headers(&parts.headers, &config.excluded_headers));
      let headers_payload = Payload::json(
        serde_json::to_string(&headers_json).unwrap_or_default(),
      );
      ObservationBuilder::new("http/request")
        .group(&http_group)
        .metadata("method", parts.method.to_string())
        .metadata("uri", parts.uri.to_string())
        .execution(&execution)
        .named_raw_payload("headers", headers_payload)
        .raw_payload("body", bytes_to_payload(&request_body_bytes, &parts.headers));

      let response = inner
        .call(Request::from_parts(parts, Body::from(request_body_bytes)))
        .await?;

      let (parts, body) = response.into_parts();
      let log_level = match parts.status.as_u16() {
        200..=299 => LogLevel::Info,
        400..=499 => LogLevel::Warning,
        500..=599 => LogLevel::Error,
        _ => LogLevel::Info,
      };

      // Single response observation with named payload "headers" sent immediately,
      // "body" added later when streaming completes via the payload handle
      let resp_headers_json = json!(filter_headers(&parts.headers, &config.excluded_headers));
      let resp_headers_payload = Payload::json(
        serde_json::to_string(&resp_headers_json).unwrap_or_default(),
      );
      let payload_handle = ObservationBuilder::new("http/response")
        .group(&http_group)
        .metadata("status", &parts.status.as_u16().to_string())
        .log_level(log_level)
        .execution(&execution)
        .named_raw_payload("headers", resp_headers_payload);

      // Wrap the response body in a streaming observer that captures data as it flows
      // through and adds the body payload when the stream completes
      let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
      let streaming_body =
        StreamingObserverBody::new(body, content_type, payload_handle);

      Ok(Response::from_parts(parts, Body::new(streaming_body)))
    })
  }
}

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

fn bytes_to_payload(bytes: &Bytes, headers: &HeaderMap) -> Payload {
  let content_type = headers
    .get(CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("application/octet-stream");

  Payload {
    data: bytes.to_vec(),
    mime_type: content_type.to_string(),
    size: bytes.len(),
  }
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
