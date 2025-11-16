//! CSRF protection middleware and utilities
//!
//! Implements double-submit cookie pattern with custom headers:
//! - Server generates a random token and stores it in a cookie
//! - Client must send the same token in the X-CSRF-Token header
//! - Server validates that cookie and header match

use axum::{
  body::Body,
  extract::{FromRequestParts, Request},
  http::{header, StatusCode},
  middleware::Next,
  response::{IntoResponse, Response},
};

/// CSRF token cookie name
pub const CSRF_COOKIE_NAME: &str = "csrf_token";

/// CSRF token header name
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// Generate a new CSRF token
pub fn generate_token() -> String {
  use uuid::Uuid;
  // Use UUID v7 for time-ordered tokens
  let uuid = Uuid::now_v7();
  // Convert to hex string for URL safety
  uuid.as_simple().to_string()
}

/// Extract CSRF token from request cookies
fn extract_cookie_token(req: &Request) -> Option<String> {
  req
    .headers()
    .get(header::COOKIE)
    .and_then(|v| v.to_str().ok())
    .and_then(|cookies| {
      cookies.split(';').find_map(|cookie| {
        let mut parts = cookie.trim().splitn(2, '=');
        match (parts.next(), parts.next()) {
          (Some(name), Some(value)) if name == CSRF_COOKIE_NAME => Some(value.to_string()),
          _ => None,
        }
      })
    })
}

/// Extract CSRF token from request header
fn extract_header_token(req: &Request) -> Option<String> {
  req
    .headers()
    .get(CSRF_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
}

/// Middleware to validate CSRF tokens on state-changing requests
///
/// This implements "lenient mode" CSRF protection:
/// - If a CSRF cookie is present, the header must also be present and match
/// - If no CSRF cookie is present, the request is allowed (for programmatic clients)
///
/// This allows:
/// 1. Browser-based requests to be protected (they receive cookies from UI pages)
/// 2. Programmatic clients (like the observation-tools-client) to work without CSRF tokens
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

  // Extract tokens from cookie and header
  let cookie_token = extract_cookie_token(&req);
  let header_token = extract_header_token(&req);

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
    // Try to get from extensions (set by UI middleware)
    if let Some(token) = parts.extensions.get::<CsrfToken>() {
      return Ok(token.clone());
    }

    // Fallback: try to extract from cookie
    let req = Request::from_parts(parts.clone(), Body::empty());
    if let Some(token) = extract_cookie_token(&req) {
      return Ok(CsrfToken(token));
    }

    // If not in cookie, generate a new one
    let token = generate_token();
    Ok(CsrfToken(token))
  }
}

/// Middleware for UI routes that automatically generates and sets CSRF tokens
///
/// This middleware:
/// - Generates or extracts a CSRF token
/// - Sets the CSRF cookie in the response
/// - Stores the token in request extensions for handlers to use
pub async fn ui_csrf_middleware(mut req: Request, next: Next) -> Response {
  // Extract or generate token
  let token = extract_cookie_token(&req).unwrap_or_else(generate_token);

  // Store token in request extensions for handlers to access
  req.extensions_mut().insert(CsrfToken(token.clone()));

  // Get response from handler
  let mut response = next.run(req).await;

  // Set CSRF cookie in response
  response
    .headers_mut()
    .insert(header::SET_COOKIE, create_csrf_cookie(&token).parse().unwrap());

  response
}

/// Helper to create a Set-Cookie header for CSRF token
pub fn create_csrf_cookie(token: &str) -> String {
  format!(
    "{}={}; Path=/; SameSite=Strict; HttpOnly",
    CSRF_COOKIE_NAME, token
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_token() {
    let token1 = generate_token();
    let token2 = generate_token();

    // Tokens should be non-empty
    assert!(!token1.is_empty());
    assert!(!token2.is_empty());

    // Tokens should be unique
    assert_ne!(token1, token2);

    // Token should be valid hex (UUID simple format is 32 hex chars)
    assert_eq!(token1.len(), 32);
    assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn test_create_csrf_cookie() {
    let token = "test-token-123";
    let cookie = create_csrf_cookie(token);

    assert!(cookie.contains("csrf_token=test-token-123"));
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("HttpOnly"));
  }
}
