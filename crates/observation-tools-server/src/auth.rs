use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use hmac::Hmac;
use hmac::Mac;
use sha2::Sha256;
use std::sync::Once;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApiKeySecret(String);

const MIN_SECRET_LENGTH: usize = 16;
pub const ENV_API_KEY_SECRET: &str = "API_KEY_SECRET";

impl ApiKeySecret {
  pub fn from_env() -> Result<Option<Self>, AuthError> {
    std::env::var(ENV_API_KEY_SECRET)
      .ok()
      .map(|s| ApiKeySecret::new(&s))
      .transpose()
  }

  pub fn new(secret: &str) -> Result<Self, AuthError> {
    if secret.len() < MIN_SECRET_LENGTH {
      return Err(AuthError::InvalidSecretLength {
        min_length: MIN_SECRET_LENGTH,
      });
    }
    Ok(Self(secret.to_string()))
  }
}

type HmacSha256 = Hmac<Sha256>;

const API_KEY_PREFIX: &str = "obs_";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
  #[error("Invalid API secret length: Must be at least {min_length} bytes")]
  InvalidSecretLength { min_length: usize },
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
      AuthError::InvalidSecretLength { .. } => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Invalid API secret configuration",
      ),
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

static API_MIDDLEWARE_INIT_ONCE: Once = Once::new();

pub async fn api_key_middleware(
  secret: Option<ApiKeySecret>,
  request: Request,
  next: Next,
) -> Result<Response, AuthError> {
  API_MIDDLEWARE_INIT_ONCE.call_once(|| {
    if let Some(_) = &secret {
      tracing::info!("API key authentication is enabled.");
    } else {
      tracing::warn!("API key authentication is disabled.");
    }
  });

  if let Some(secret) = secret {
    let auth_header = request
      .headers()
      .get(AUTHORIZATION)
      .ok_or(AuthError::MissingAuthHeader)?
      .to_str()
      .map_err(|_| AuthError::InvalidAuthFormat)?;
    let Some(token) = auth_header.strip_prefix("Bearer ") else {
      return Err(AuthError::InvalidAuthFormat);
    };
    validate_api_key(&token, &secret)?;
  }
  Ok(next.run(request).await)
}

fn validate_api_key(api_key: &str, secret: &ApiKeySecret) -> Result<(), AuthError> {
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
    HmacSha256::new_from_slice(secret.0.as_bytes()).map_err(|_| AuthError::HmacInitFailed)?;
  mac.update(uuid_bytes);
  mac
    .verify_slice(provided_signature)
    .map_err(|_| AuthError::InvalidApiKey)?;

  Ok(())
}

pub fn generate_api_key(secret: &ApiKeySecret) -> Result<String, AuthError> {
  let uuid = Uuid::new_v4();
  let uuid_bytes = uuid.as_bytes();
  let mut mac =
    HmacSha256::new_from_slice(secret.0.as_bytes()).map_err(|_| AuthError::HmacInitFailed)?;
  mac.update(uuid_bytes);
  let signature = mac.finalize().into_bytes();
  let mut payload = Vec::new();
  payload.extend_from_slice(uuid_bytes);
  payload.extend_from_slice(&signature);
  Ok(format!(
    "{}{}",
    API_KEY_PREFIX,
    URL_SAFE_NO_PAD.encode(&payload)
  ))
}
