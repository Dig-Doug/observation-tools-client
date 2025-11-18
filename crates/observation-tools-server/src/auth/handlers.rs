//! Authentication HTTP handlers

use super::middleware::{AuthUser, SESSION_COOKIE_NAME};
use super::models::{Provider, Session, SocialAccount, User};
use super::oauth::{OAuthManager, OAuthUserInfo};
use super::storage::AuthStorage;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use time::Duration;
use std::sync::Arc;
use tracing::{error, info};

/// Shared application state for auth handlers
#[derive(Clone)]
pub struct AuthState {
    pub auth_storage: Arc<dyn AuthStorage>,
    pub oauth_manager: Arc<OAuthManager>,
}

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: Option<String>,
}

/// Handler for login page
pub async fn login_page() -> Response {
    // For now, just redirect to home (which will show login buttons)
    Redirect::to("/").into_response()
}

/// Handler to initiate OAuth flow
pub async fn oauth_login(
    State(state): State<AuthState>,
    Path(provider_str): Path<String>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), Response> {
    // Parse provider
    let provider: Provider = provider_str
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid provider").into_response())?;

    // Get OAuth provider
    let oauth_provider = state
        .oauth_manager
        .get_provider(provider)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Provider not configured").into_response())?;

    // Generate authorization URL
    let (auth_url, csrf_token) = oauth_provider.authorize_url();

    info!("Initiating OAuth login for provider: {}", provider);

    // Store CSRF token in cookie
    let csrf_cookie = Cookie::build((format!("oauth_csrf_{}", provider.as_str()), csrf_token.secret().clone()))
        .path("/")
        .http_only(true)
        .max_age(Duration::minutes(10))
        .build();

    let jar = jar.add(csrf_cookie);

    Ok((jar, Redirect::to(&auth_url)))
}

/// Handler for OAuth callback
pub async fn oauth_callback(
    State(state): State<AuthState>,
    Path(provider_str): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), Response> {
    // Parse provider
    let provider: Provider = provider_str
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid provider").into_response())?;

    // Verify CSRF token
    let csrf_cookie_name = format!("oauth_csrf_{}", provider.as_str());
    let stored_csrf = jar
        .get(&csrf_cookie_name)
        .map(|c| c.value())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing CSRF token").into_response())?;

    if let Some(state_token) = &query.state {
        if state_token != stored_csrf {
            error!("CSRF token mismatch");
            return Err((StatusCode::BAD_REQUEST, "Invalid CSRF token").into_response());
        }
    }

    // Remove CSRF cookie
    let jar = jar.remove(Cookie::from(csrf_cookie_name));

    // Get OAuth provider
    let oauth_provider = state
        .oauth_manager
        .get_provider(provider)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Provider not configured").into_response())?;

    // Exchange code for token and get user info
    let (access_token, user_info) = oauth_provider
        .exchange_code(query.code)
        .await
        .map_err(|e| {
            error!("Failed to exchange OAuth code: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "OAuth exchange failed").into_response()
        })?;

    info!(
        "OAuth callback successful for provider: {}, user: {}",
        provider, user_info.email
    );

    // Get or create user
    let user = get_or_create_user(&state, provider, &user_info, &access_token)
        .await
        .map_err(|e| {
            error!("Failed to get or create user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create user",
            )
                .into_response()
        })?;

    // Create session
    let session = Session::new(user.id, 30); // 30 days
    state
        .auth_storage
        .store_session(&session)
        .await
        .map_err(|e| {
            error!("Failed to store session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create session",
            )
                .into_response()
        })?;

    info!("Session created for user: {}", user.email);

    // Set session cookie
    let session_cookie = Cookie::build((SESSION_COOKIE_NAME, session.id.clone()))
        .path("/")
        .http_only(true)
        .max_age(Duration::days(30))
        .build();

    let jar = jar.add(session_cookie);

    Ok((jar, Redirect::to("/")))
}

/// Handler for logout
pub async fn logout(
    State(state): State<AuthState>,
    Extension(auth_user): Extension<AuthUser>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), Response> {
    // Delete session
    state
        .auth_storage
        .delete_session(&auth_user.session.id)
        .await
        .map_err(|e| {
            error!("Failed to delete session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to logout",
            )
                .into_response()
        })?;

    info!("User logged out: {}", auth_user.user.email);

    // Remove session cookie
    let jar = jar.remove(Cookie::from(SESSION_COOKIE_NAME));

    Ok((jar, Redirect::to("/")))
}

/// Get or create user from OAuth info
async fn get_or_create_user(
    state: &AuthState,
    provider: Provider,
    user_info: &OAuthUserInfo,
    access_token: &str,
) -> anyhow::Result<User> {
    // Check if social account exists
    if let Some(social_account) = state
        .auth_storage
        .get_social_account(provider, &user_info.provider_user_id)
        .await?
    {
        // Get existing user
        if let Some(user) = state.auth_storage.get_user(social_account.user_id).await? {
            // Update social account with new token
            let updated_account = SocialAccount {
                access_token: Some(access_token.to_string()),
                updated_at: chrono::Utc::now(),
                ..social_account
            };
            state.auth_storage.store_social_account(&updated_account).await?;
            return Ok(user);
        }
    }

    // Check if user exists with same email
    let user = if let Some(existing_user) = state.auth_storage.get_user_by_email(&user_info.email).await? {
        existing_user
    } else {
        // Create new user
        let new_user = User::new(
            user_info.email.clone(),
            user_info.name.clone(),
            user_info.avatar_url.clone(),
        );
        state.auth_storage.store_user(&new_user).await?;
        new_user
    };

    // Create or update social account
    let social_account = SocialAccount::new(
        user.id,
        provider,
        user_info.provider_user_id.clone(),
        user_info.email.clone(),
        user_info.name.clone(),
        user_info.avatar_url.clone(),
        Some(access_token.to_string()),
        None,
    );

    state.auth_storage.store_social_account(&social_account).await?;

    Ok(user)
}
