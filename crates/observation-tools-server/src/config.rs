//! Server configuration

use std::net::SocketAddr;
use std::path::PathBuf;

/// OAuth provider configuration
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret: String,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
  /// Address to bind the server to
  pub bind_addr: SocketAddr,

  /// Directory for storing data
  pub data_dir: PathBuf,

  /// Directory for storing blobs
  pub blob_dir: PathBuf,

  /// Base URL for the server (used for OAuth redirects)
  pub base_url: String,

  /// Google OAuth configuration
  pub google_oauth: Option<OAuthProviderConfig>,

  /// GitHub OAuth configuration
  pub github_oauth: Option<OAuthProviderConfig>,
}

impl Config {
  /// Create a new configuration with default values
  pub fn new() -> Self {
    let data_dir = PathBuf::from(".observation-tools");
    let blob_dir = data_dir.join("blobs");

    Self {
      bind_addr: "127.0.0.1:3000".parse().unwrap(),
      data_dir,
      blob_dir,
      base_url: "http://localhost:3000".to_string(),
      google_oauth: None,
      github_oauth: None,
    }
  }

  /// Set the bind address
  pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
    self.bind_addr = addr;
    self
  }

  /// Set the data directory
  pub fn with_data_dir(mut self, dir: PathBuf) -> Self {
    self.data_dir = dir.clone();
    self.blob_dir = dir.join("blobs");
    self
  }

  /// Set the base URL
  pub fn with_base_url(mut self, url: String) -> Self {
    self.base_url = url;
    self
  }

  /// Set Google OAuth configuration
  pub fn with_google_oauth(mut self, client_id: String, client_secret: String) -> Self {
    self.google_oauth = Some(OAuthProviderConfig {
      client_id,
      client_secret,
    });
    self
  }

  /// Set GitHub OAuth configuration
  pub fn with_github_oauth(mut self, client_id: String, client_secret: String) -> Self {
    self.github_oauth = Some(OAuthProviderConfig {
      client_id,
      client_secret,
    });
    self
  }
}

impl Default for Config {
  fn default() -> Self {
    Self::new()
  }
}
