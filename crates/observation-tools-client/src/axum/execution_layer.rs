use crate::context::with_execution;
use crate::Client;
use axum::extract::Request;
use axum::response::Response;
use http::Method;
use http::Uri;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tower::Layer;
use tower::Service;

/// Layer that creates an execution context for each request
#[derive(Clone)]
pub struct ExecutionLayer {
  client: Arc<Client>,
}

impl ExecutionLayer {
  /// Create a new ExecutionLayer with the given client
  pub fn new(client: Client) -> Self {
    Self {
      client: Arc::new(client),
    }
  }
}

impl<S> Layer<S> for ExecutionLayer {
  type Service = ExecutionService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    ExecutionService {
      inner,
      client: self.client.clone(),
    }
  }
}

/// Service that wraps requests with an execution context
#[derive(Clone)]
pub struct ExecutionService<S> {
  inner: S,
  client: Arc<Client>,
}

impl<S> Service<Request> for ExecutionService<S>
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
    let client = self.client.clone();
    let mut inner = self.inner.clone();

    Box::pin(async move {
      let execution =
        match client.begin_execution(&format!("{} {}", req.method(), req.uri().path())) {
          Ok(begin) => begin.into_handle(),
          Err(e) => {
            tracing::error!("Failed to create execution: {}", e);
            // Continue without execution context - observations will fail
            // but the request will still be processed
            return inner.call(req).await;
          }
        };
      with_execution(execution, inner.call(req)).await
    })
  }
}
