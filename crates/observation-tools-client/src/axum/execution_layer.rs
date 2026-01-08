use crate::context::with_execution;
use crate::Client;
use axum::extract::Request;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tower::Layer;
use tower::Service;

/// A filter function that determines whether an execution should be created for
/// a request.
///
/// Returns `true` to create an execution, `false` to skip.
pub type RequestFilter = Arc<dyn Fn(&Request) -> bool + Send + Sync>;

/// Layer that creates an execution context for each request
#[derive(Clone)]
pub struct ExecutionLayer {
  client: Arc<Client>,
  filter: Option<RequestFilter>,
}

impl ExecutionLayer {
  /// Create a new ExecutionLayer with the given client
  pub fn new(client: Client) -> Self {
    Self {
      client: Arc::new(client),
      filter: None,
    }
  }

  /// Set a filter function that determines whether an execution should be
  /// created.
  ///
  /// The filter receives a reference to the incoming request and returns `true`
  /// to create an execution context, or `false` to skip execution creation.
  /// When skipped, the request proceeds without an execution context.
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// ExecutionLayer::new(client)
  ///     .with_filter(|req| {
  ///         // Skip health check endpoints
  ///         !req.uri().path().starts_with("/health")
  ///     })
  /// ```
  pub fn with_filter<F>(mut self, filter: F) -> Self
  where
    F: Fn(&Request) -> bool + Send + Sync + 'static,
  {
    self.filter = Some(Arc::new(filter));
    self
  }
}

impl<S> Layer<S> for ExecutionLayer {
  type Service = ExecutionService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    ExecutionService {
      inner,
      client: self.client.clone(),
      filter: self.filter.clone(),
    }
  }
}

/// Service that wraps requests with an execution context
#[derive(Clone)]
pub struct ExecutionService<S> {
  inner: S,
  client: Arc<Client>,
  filter: Option<RequestFilter>,
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
    let filter = self.filter.clone();
    let mut inner = self.inner.clone();

    Box::pin(async move {
      // Check filter if provided
      if let Some(ref filter) = filter {
        if !filter(&req) {
          // Filter returned false, skip execution creation
          return inner.call(req).await;
        }
      }

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
