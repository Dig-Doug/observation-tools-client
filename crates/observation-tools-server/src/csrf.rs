use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::header;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use uuid::Uuid;

pub const CSRF_COOKIE_NAME: &str = "csrf_token";
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// Middleware to validate CSRF tokens using the [naive double-submit cookie pattern](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#naive-double-submit-cookie-pattern-discouraged)
///
/// TODO(doug): Upgrade to signed double-submit cookies
/// TODO(doug): Verify origin/header for additional security
pub async fn validate_csrf(req: Request, next: Next) -> Response {
  // Only validate POST, PUT, DELETE, PATCH methods
  let method = req.method();
  if !matches!(
    method,
    &axum::http::Method::POST
      | &axum::http::Method::PUT
      | &axum::http::Method::DELETE
      | &axum::http::Method::PATCH
  ) {
    return next.run(req).await;
  }

  let cookie_token = req
    .headers()
    .get(header::COOKIE)
    .and_then(|v| v.to_str().ok())
    .and_then(|cookies_str| {
      Cookie::split_parse(cookies_str)
        .flatten()
        .find(|cookie| cookie.name() == CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
    });
  let header_token = req
    .headers()
    .get(CSRF_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string());

  // Lenient mode: only validate if cookie is present
  match (cookie_token, header_token) {
    // No cookie = programmatic client, allow it
    (None, _) => {
      tracing::debug!("CSRF validation skipped: no cookie present (programmatic client)");
      next.run(req).await
    }
    // Cookie present but no header = potential CSRF attack
    (Some(_), None) => {
      tracing::warn!("CSRF validation failed: cookie present but missing header token");
      (
        StatusCode::FORBIDDEN,
        "CSRF token missing in request header. Please include X-CSRF-Token header.",
      )
        .into_response()
    }
    // Both present, check they match
    (Some(cookie), Some(header)) if cookie == header => {
      tracing::debug!("CSRF token validated successfully");
      next.run(req).await
    }
    // Both present but don't match = potential CSRF attack
    (Some(cookie), Some(header)) => {
      tracing::warn!(
        "CSRF validation failed: token mismatch (cookie length: {}, header length: {})",
        cookie.len(),
        header.len()
      );
      (
        StatusCode::FORBIDDEN,
        "CSRF token mismatch. Please refresh the page and try again.",
      )
        .into_response()
    }
  }
}

/// CSRF token extractor for use in handlers
///
/// This can be used to access the current CSRF token in UI handlers
#[derive(Debug, Clone)]
pub struct CsrfToken(pub String);

impl<S> FromRequestParts<S> for CsrfToken
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    parts
      .extensions
      .get::<CsrfToken>()
      .cloned()
      .ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "CSRF token not found in request extensions. Ensure ui_csrf_middleware is applied to this route.",
      ))
  }
}

/// Middleware for UI routes that automatically generates and sets CSRF tokens
pub async fn ui_csrf_middleware(mut req: Request, next: Next) -> Response {
  let token = Uuid::new_v4().as_simple().to_string();
  req.extensions_mut().insert(CsrfToken(token.clone()));
  let mut response = next.run(req).await;
  let cookie = Cookie::build((CSRF_COOKIE_NAME, token.to_string()))
    .path("/")
    .same_site(SameSite::Strict)
    .http_only(true)
    .build();
  response.headers_mut().insert(
    header::SET_COOKIE,
    cookie.to_string().parse().expect("valid cookie"),
  );
  response
}
