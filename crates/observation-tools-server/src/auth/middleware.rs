//! Authentication middleware

use super::models::{Session, User};
use super::storage::AuthStorage;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

/// Session cookie name
pub const SESSION_COOKIE_NAME: &str = "session_id";

/// Authenticated user extracted from session
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub session: Session,
}

/// Extract authenticated user from session cookie
pub async fn extract_user<S>(
    State(auth_storage): State<Arc<dyn AuthStorage>>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        let session_id = session_cookie.value().to_string();

        // Get session
        if let Ok(Some(session)) = auth_storage.get_session(&session_id).await {
            // Get user
            if let Ok(Some(user)) = auth_storage.get_user(session.user_id).await {
                let auth_user = AuthUser { user, session };
                request.extensions_mut().insert(auth_user);
            }
        }
    }

    next.run(request).await
}

/// Require authenticated user (returns 401 if not authenticated)
pub async fn require_auth(request: Request, next: Next) -> Response {
    if request.extensions().get::<AuthUser>().is_none() {
        return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    }

    next.run(request).await
}
