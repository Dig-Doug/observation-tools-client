//! Authentication data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a user
pub type UserId = Uuid;

/// Unique identifier for a session
pub type SessionId = String;

/// Social authentication provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    GitHub,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Google => "google",
            Provider::GitHub => "github",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(Provider::Google),
            "github" => Ok(Provider::GitHub),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, name: Option<String>, avatar_url: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            name,
            avatar_url,
            created_at: Utc::now(),
        }
    }
}

/// Social account linked to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccount {
    pub user_id: UserId,
    pub provider: Provider,
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SocialAccount {
    pub fn new(
        user_id: UserId,
        provider: Provider,
        provider_user_id: String,
        email: String,
        name: Option<String>,
        avatar_url: Option<String>,
        access_token: Option<String>,
        refresh_token: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            provider,
            provider_user_id,
            email,
            name,
            avatar_url,
            access_token,
            refresh_token,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the unique key for this social account (provider:provider_user_id)
    pub fn key(&self) -> String {
        format!("{}:{}", self.provider.as_str(), self.provider_user_id)
    }
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: UserId, duration_days: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::days(duration_days),
        }
    }

    /// Generate a secure random session ID
    fn generate_id() -> SessionId {
        use rand::Rng;
        let random_bytes: [u8; 32] = rand::thread_rng().gen();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, random_bytes)
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
