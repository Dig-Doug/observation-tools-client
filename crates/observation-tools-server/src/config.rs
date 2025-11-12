//! Server configuration

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
}

impl Default for Config {
  fn default() -> Self {
    Self::new()
  }
}
