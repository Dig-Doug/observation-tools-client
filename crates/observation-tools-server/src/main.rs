//! Observation Tools Server
//!
//! HTTP API backend that collects, stores, indexes, and serves logged
//! observations.

use observation_tools_server::Config;
use observation_tools_server::Server;
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Initialize tracing
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .pretty()
    .init();

  // Read configuration from environment variables
  let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
  let port = std::env::var("PORT")
    .ok()
    .and_then(|p| p.parse::<u16>().ok())
    .unwrap_or(3000);

  let data_dir = std::env::var("DATA_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from(".observation-tools"));

  let bind_addr: SocketAddr = format!("{}:{}", host, port)
    .parse()
    .expect("Invalid HOST or PORT");

  tracing::info!("Starting server with configuration:");
  tracing::info!("  Bind address: {}", bind_addr);
  tracing::info!("  Data directory: {}", data_dir.display());

  let config = Config::new()
    .with_bind_addr(bind_addr)
    .with_data_dir(data_dir);

  let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
  let server = Server::new(config);
  server.run(listener).await?;

  Ok(())
}
