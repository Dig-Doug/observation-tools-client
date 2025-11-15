use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

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
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingAuthHeader => (StatusCode::UNAUTHORIZED, "Missing Authorization header"),
            AuthError::InvalidAuthFormat => (StatusCode::UNAUTHORIZED, "Invalid Authorization header format"),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key"),
        };

        (status, message).into_response()
    }
}

pub fn generate_api_key(secret: &str) -> String {
    let mut rng = rand::thread_rng();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let random_bytes: [u8; 16] = rng.gen();

    let mut payload = Vec::new();
    payload.extend_from_slice(&timestamp.to_be_bytes());
    payload.extend_from_slice(&random_bytes);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&payload);
    let signature = mac.finalize().into_bytes();

    payload.extend_from_slice(&signature);

    format!("{}{}", API_KEY_PREFIX, URL_SAFE_NO_PAD.encode(&payload))
}

pub fn validate_api_key(api_key: &str, secret: &str) -> Result<(), AuthError> {
    if !api_key.starts_with(API_KEY_PREFIX) {
        return Err(AuthError::InvalidApiKey);
    }

    let encoded = &api_key[API_KEY_PREFIX.len()..];
    let payload = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| AuthError::InvalidApiKey)?;

    if payload.len() < 8 + 16 + 32 {
        return Err(AuthError::InvalidApiKey);
    }

    let data = &payload[..8 + 16];
    let provided_signature = &payload[8 + 16..];

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data);

    mac.verify_slice(provided_signature)
        .map_err(|_| AuthError::InvalidApiKey)?;

    Ok(())
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
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
    secret: String,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = extract_bearer_token(request.headers())?;
    validate_api_key(&token, &secret)?;
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_api_key() {
        let secret = "test-secret-key";
        let api_key = generate_api_key(secret);

        assert!(api_key.starts_with(API_KEY_PREFIX));
        assert!(validate_api_key(&api_key, secret).is_ok());
    }

    #[test]
    fn test_invalid_api_key() {
        let secret = "test-secret-key";
        let api_key = generate_api_key(secret);

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
