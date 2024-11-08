use crate::auth::AuthState;
use axum::async_trait;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Principal {
    Anonymous,
}

#[async_trait]
impl<S> FromRequestParts<S> for Principal
where
    AuthState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = parts
            .extract_with_state::<AuthState, _>(state)
            .await
            .map_err(|_e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to extract auth state".to_string(),
                )
            })?;
        // TODO(doug): Implement authentication
        Ok(Principal::Anonymous)
    }
}
