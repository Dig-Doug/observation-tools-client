use axum::extract::Request;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use hmac::Hmac;
use hmac::Mac;
use sha2::Sha256;
use uuid::Uuid;
use axum::http::header::AUTHORIZATION;

type HmacSha256 = Hmac<Sha256>;

const API_KEY_PREFIX: &str = "obs_";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
  #[error("Missing Authorization header")]
  MissingAuthHeader,
  #[error("Invalid Authorization header format")]
  InvalidAuthFormat,
  #[error("Invalid API key")]
  InvalidApiKey,
  #[error("HMAC initialization failed")]
  HmacInitFailed,
}

impl IntoResponse for AuthError {
  fn into_response(self) -> Response {
    let (status, message) = match self {
      AuthError::MissingAuthHeader => (StatusCode::UNAUTHORIZED, "Missing Authorization header"),
      AuthError::InvalidAuthFormat => (
        StatusCode::UNAUTHORIZED,
        "Invalid Authorization header format",
      ),
      AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key"),
      AuthError::HmacInitFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    };

    (status, message).into_response()
  }
}

pub fn generate_api_key(secret: &str) -> Result<String, AuthError> {
  let uuid = Uuid::new_v4();
  let uuid_bytes = uuid.as_bytes();

  let mut mac =
    HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AuthError::HmacInitFailed)?;
  mac.update(uuid_bytes);
  let signature = mac.finalize().into_bytes();

  let mut payload = Vec::new();
  payload.extend_from_slice(uuid_bytes);
  payload.extend_from_slice(&signature);

  Ok(format!("{}{}", API_KEY_PREFIX, URL_SAFE_NO_PAD.encode(&payload)))
}

pub fn validate_api_key(api_key: &str, secret: &str) -> Result<(), AuthError> {
  if !api_key.starts_with(API_KEY_PREFIX) {
    return Err(AuthError::InvalidApiKey);
  }

  let encoded = &api_key[API_KEY_PREFIX.len()..];
  let payload = URL_SAFE_NO_PAD
    .decode(encoded)
    .map_err(|_| AuthError::InvalidApiKey)?;

  if payload.len() < 16 + 32 {
    return Err(AuthError::InvalidApiKey);
  }

  let uuid_bytes = &payload[..16];
  let provided_signature = &payload[16..];

  let mut mac =
    HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AuthError::HmacInitFailed)?;
  mac.update(uuid_bytes);

  mac
    .verify_slice(provided_signature)
    .map_err(|_| AuthError::InvalidApiKey)?;

  Ok(())
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthError> {
  let auth_header = headers
    .get(AUTHORIZATION)
    .ok_or(AuthError::MissingAuthHeader)?
    .to_str()
    .map_err(|_| AuthError::InvalidAuthFormat)?;

  if let Some(token) = auth_header.strip_prefix("Bearer ") {
    Ok(token.to_string())
  } else {
    Err(AuthError::InvalidAuthFormat)
  }
}

pub async fn api_key_middleware(
  secret: Option<String>,
  request: Request,
  next: Next,
) -> Result<Response, AuthError> {
  if let Some(secret) = secret {
    let token = extract_bearer_token(request.headers())?;
    validate_api_key(&token, &secret)?;
  }
  Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_and_validate_api_key() {
    let secret = "test-secret-key";
    let api_key = generate_api_key(secret).unwrap();

    assert!(api_key.starts_with(API_KEY_PREFIX));
    assert!(validate_api_key(&api_key, secret).is_ok());
  }

  #[test]
  fn test_invalid_api_key() {
    let secret = "test-secret-key";
    let api_key = generate_api_key(secret).unwrap();

    let wrong_secret = "wrong-secret";
    assert!(validate_api_key(&api_key, wrong_secret).is_err());
  }

  #[test]
  fn test_malformed_api_key() {
    let secret = "test-secret-key";
    assert!(validate_api_key("invalid-key", secret).is_err());
    assert!(validate_api_key("obs_invalid", secret).is_err());
  }
}
