//! Server configuration

use crate::auth::ApiKeySecret;
use directories::ProjectDirs;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
  /// Address to bind the server to
  pub bind_addr: SocketAddr,

  /// Directory for storing data
  pub data_dir: PathBuf,

  /// Directory for storing blobs
  pub blob_dir: PathBuf,

  /// Optional API secret for authentication
  pub api_secret: Option<ApiKeySecret>,
}

impl Config {
  /// Create a new configuration with default values
  pub fn new() -> Self {
    let data_dir = ProjectDirs::from("", "", "observation-tools")
      .map(|dirs| dirs.data_dir().to_path_buf())
      .unwrap_or_else(|| PathBuf::from(".observation-tools"));
    let blob_dir = data_dir.join("blobs");

    Self {
      bind_addr: "127.0.0.1:3000".parse().unwrap(),
      data_dir,
      blob_dir,
      api_secret: None,
    }
  }

  /// Set the bind address
  pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
    self.bind_addr = addr;
    self
  }

  /// Set the data directory. If `None`, keeps the default.
  pub fn with_data_dir(mut self, dir: Option<PathBuf>) -> Self {
    if let Some(dir) = dir {
      self.data_dir = dir.clone();
      self.blob_dir = dir.join("blobs");
    }
    self
  }

  /// Set the API secret for authentication
  pub fn with_api_secret(mut self, secret: Option<ApiKeySecret>) -> Self {
    self.api_secret = secret;
    self
  }
}

impl Default for Config {
  fn default() -> Self {
    Self::new()
  }
}
