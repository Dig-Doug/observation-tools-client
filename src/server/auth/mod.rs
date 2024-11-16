use axum::async_trait;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::convert::Infallible;

pub mod permission;
pub mod principal;
pub mod resource_id;

#[derive(Clone)]
pub struct AuthState {}

#[async_trait]
impl<S> FromRequestParts<S> for AuthState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}
