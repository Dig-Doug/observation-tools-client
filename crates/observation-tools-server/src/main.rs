//! Observation Tools Server
//!
//! HTTP API backend that collects, stores, indexes, and serves logged
//! observations.

use clap::Parser;
use observation_tools_server::Config;
use observation_tools_server::Server;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "observation-tools")]
#[command(about = "Observation Tools Server", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
  /// Start the observation tools server
  Serve {
    /// Directory for storing data
    #[arg(short, long, default_value = ".observation-tools")]
    data_dir: PathBuf,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Load environment variables from .env file if it exists
  if let Err(e) = dotenvy::dotenv() {
    // Ignore the error if the file doesn't exist (NotFound)
    if !e.to_string().contains("No such file or directory") {
      eprintln!("Warning: Failed to load .env file: {}", e);
    }
  }

  // Initialize tracing
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .pretty()
    .init();

  let cli = Cli::parse();

  match cli.command {
    Commands::Serve { data_dir } => {
      // Read PORT from environment or use default
      let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

      let bind_addr: SocketAddr = ([0, 0, 0, 0], port).into();

      // Read BASE_URL from environment
      let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| format!("http://localhost:{}", port));

      let mut config = Config::new()
        .with_bind_addr(bind_addr)
        .with_data_dir(data_dir)
        .with_base_url(base_url);

      // Configure Google OAuth if credentials are provided
      if let (Ok(client_id), Ok(client_secret)) = (
        std::env::var("GOOGLE_CLIENT_ID"),
        std::env::var("GOOGLE_CLIENT_SECRET"),
      ) {
        tracing::info!("Configuring Google OAuth");
        config = config.with_google_oauth(client_id, client_secret);
      }

      // Configure GitHub OAuth if credentials are provided
      if let (Ok(client_id), Ok(client_secret)) = (
        std::env::var("GITHUB_CLIENT_ID"),
        std::env::var("GITHUB_CLIENT_SECRET"),
      ) {
        tracing::info!("Configuring GitHub OAuth");
        config = config.with_github_oauth(client_id, client_secret);
      }

      let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
      let server = Server::new(config);
      server.run(listener).await?;
    }
  }

  Ok(())
}
