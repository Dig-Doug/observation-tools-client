use futures::ready;
use pin_project::pin_project;
use reqwest::cookie;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tower::limit::concurrency::future::ResponseFuture;
use tower::limit::ConcurrencyLimit;
use tower::Layer;
use tower_service::Service;
use tracing::trace;
use url::Url;

pub(crate) struct ThrottleWithoutAccessCookieLayer {
    pub cookie_store: Arc<dyn cookie::CookieStore>,
    pub api_host: Url,
}

impl<S> Layer<S> for ThrottleWithoutAccessCookieLayer
    where
        S: Clone,
{
    type Service = ThrottleWithoutAccessCookie<S>;

    fn layer(&self, service: S) -> Self::Service {
        ThrottleWithoutAccessCookie::new(service, self.cookie_store.clone(), self.api_host.clone())
    }
}

/// Throttle requests until we get an access token cookie, which makes requests
/// cheaper and faster.
#[derive(Clone)]
pub(crate) struct ThrottleWithoutAccessCookie<T> {
    inner: T,
    cookie_store: Arc<dyn cookie::CookieStore>,
    api_host: Url,
    concurrency_limiter: ConcurrencyLimit<T>,
}

impl<T> ThrottleWithoutAccessCookie<T>
    where
        T: Clone,
{
    /// Create a new concurrency limiter.
    pub fn new(inner: T, cookie_store: Arc<dyn cookie::CookieStore>, api_host: Url) -> Self {
        ThrottleWithoutAccessCookie {
            inner: inner.clone(),
            cookie_store,
            api_host,
            concurrency_limiter: ConcurrencyLimit::new(inner, 1),
        }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner service
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }

    fn has_access_token_cookie(&self) -> bool {
        self.cookie_store
            .cookies(&self.api_host)
            .and_then(|cookie_header| {
                cookie_header
                    .to_str()
                    .ok()
                    .map(|h| h.contains(ACCESS_TOKEN_COOKIE))
            })
            .unwrap_or_default()
    }
}

const ACCESS_TOKEN_COOKIE: &str = "ObsToolsAccessToken";

impl<S, Request> Service<Request> for ThrottleWithoutAccessCookie<S>
    where
        S: Service<Request> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ThrottleWithoutAccessCookieFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.has_access_token_cookie() {
            self.inner.poll_ready(cx)
        } else {
            self.concurrency_limiter.poll_ready(cx)
        }
    }

    fn call(&mut self, request: Request) -> Self::Future {
        if self.has_access_token_cookie() {
            ThrottleWithoutAccessCookieFuture::InnerFuture(self.inner.call(request))
        } else {
            ThrottleWithoutAccessCookieFuture::ResponseFuture(
                self.concurrency_limiter.call(request),
            )
        }
    }
}

#[pin_project(project = ThrottleWithoutAccessCookieFutureProj)]
pub enum ThrottleWithoutAccessCookieFuture<T> {
    InnerFuture(#[pin] T),
    ResponseFuture(#[pin] ResponseFuture<T>),
}

impl<F, T, E> Future for ThrottleWithoutAccessCookieFuture<F>
    where
        F: Future<Output=Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ThrottleWithoutAccessCookieFutureProj::InnerFuture(future) => {
                // TODO(doug): Do we need to wrap in Ready(ready!())?
                Poll::Ready(ready!(future.poll(cx)))
            }
            ThrottleWithoutAccessCookieFutureProj::ResponseFuture(future) => {
                Poll::Ready(ready!(future.poll(cx)))
            }
        }
    }
}
