//! OAuth provider implementations

use super::models::Provider;
use anyhow::{anyhow, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth configuration for a provider
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

/// User information returned from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// OAuth client wrapper
pub struct OAuthProvider {
    provider: Provider,
    client: BasicClient,
    http_client: HttpClient,
}

impl OAuthProvider {
    /// Create a new OAuth provider
    pub fn new(provider: Provider, config: OAuthConfig) -> Result<Self> {
        let (auth_url, token_url, _userinfo_url) = Self::get_provider_urls(provider)?;

        let client = BasicClient::new(
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
            AuthUrl::new(auth_url)?,
            Some(TokenUrl::new(token_url)?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url)?);

        Ok(Self {
            provider,
            client,
            http_client: HttpClient::new(),
        })
    }

    /// Get provider-specific URLs
    fn get_provider_urls(provider: Provider) -> Result<(String, String, String)> {
        match provider {
            Provider::Google => Ok((
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                "https://oauth2.googleapis.com/token".to_string(),
                "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            )),
            Provider::GitHub => Ok((
                "https://github.com/login/oauth/authorize".to_string(),
                "https://github.com/login/oauth/access_token".to_string(),
                "https://api.github.com/user".to_string(),
            )),
        }
    }

    /// Get provider-specific scopes
    fn get_scopes(&self) -> Vec<Scope> {
        match self.provider {
            Provider::Google => vec![
                Scope::new("openid".to_string()),
                Scope::new("email".to_string()),
                Scope::new("profile".to_string()),
            ],
            Provider::GitHub => vec![
                Scope::new("read:user".to_string()),
                Scope::new("user:email".to_string()),
            ],
        }
    }

    /// Generate authorization URL
    pub fn authorize_url(&self) -> (String, CsrfToken) {
        let mut auth_request = self.client.authorize_url(CsrfToken::new_random);

        for scope in self.get_scopes() {
            auth_request = auth_request.add_scope(scope);
        }

        let (url, csrf_token) = auth_request.url();
        (url.to_string(), csrf_token)
    }

    /// Exchange authorization code for access token and fetch user info
    pub async fn exchange_code(&self, code: String) -> Result<(String, OAuthUserInfo)> {
        // Exchange code for token
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| anyhow!("Failed to exchange code: {}", e))?;

        let access_token = token_result.access_token().secret().to_string();

        // Fetch user info
        let user_info = self.fetch_user_info(&access_token).await?;

        Ok((access_token, user_info))
    }

    /// Fetch user information from the provider
    async fn fetch_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let (_auth_url, _token_url, userinfo_url) = Self::get_provider_urls(self.provider)?;

        let response = self
            .http_client
            .get(&userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch user info: {}",
                response.status()
            ));
        }

        let user_data: serde_json::Value = response.json().await?;

        match self.provider {
            Provider::Google => self.parse_google_user_info(user_data),
            Provider::GitHub => self.parse_github_user_info(access_token).await,
        }
    }

    /// Parse Google user info
    fn parse_google_user_info(&self, data: serde_json::Value) -> Result<OAuthUserInfo> {
        Ok(OAuthUserInfo {
            provider_user_id: data["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing user ID"))?
                .to_string(),
            email: data["email"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing email"))?
                .to_string(),
            name: data["name"].as_str().map(|s| s.to_string()),
            avatar_url: data["picture"].as_str().map(|s| s.to_string()),
        })
    }

    /// Parse GitHub user info (requires additional API calls)
    async fn parse_github_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        // Get user profile
        let user_response = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "observation-tools")
            .send()
            .await?;

        if !user_response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch GitHub user: {}",
                user_response.status()
            ));
        }

        let user_data: serde_json::Value = user_response.json().await?;

        // Get primary email if not public
        let email = if let Some(email) = user_data["email"].as_str() {
            email.to_string()
        } else {
            // Fetch emails endpoint
            let emails_response = self
                .http_client
                .get("https://api.github.com/user/emails")
                .bearer_auth(access_token)
                .header("User-Agent", "observation-tools")
                .send()
                .await?;

            if !emails_response.status().is_success() {
                return Err(anyhow!("Failed to fetch GitHub emails"));
            }

            let emails: Vec<serde_json::Value> = emails_response.json().await?;
            emails
                .iter()
                .find(|e| e["primary"].as_bool().unwrap_or(false))
                .and_then(|e| e["email"].as_str())
                .ok_or_else(|| anyhow!("No primary email found"))?
                .to_string()
        };

        Ok(OAuthUserInfo {
            provider_user_id: user_data["id"]
                .as_u64()
                .ok_or_else(|| anyhow!("Missing user ID"))?
                .to_string(),
            email,
            name: user_data["name"].as_str().map(|s| s.to_string()),
            avatar_url: user_data["avatar_url"].as_str().map(|s| s.to_string()),
        })
    }
}

/// OAuth provider manager
pub struct OAuthManager {
    providers: HashMap<Provider, OAuthProvider>,
}

impl OAuthManager {
    /// Create a new OAuth manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: Provider, config: OAuthConfig) -> Result<()> {
        let oauth_provider = OAuthProvider::new(provider, config)?;
        self.providers.insert(provider, oauth_provider);
        Ok(())
    }

    /// Get a provider
    pub fn get_provider(&self, provider: Provider) -> Option<&OAuthProvider> {
        self.providers.get(&provider)
    }
}

impl Default for OAuthManager {
    fn default() -> Self {
        Self::new()
    }
}
