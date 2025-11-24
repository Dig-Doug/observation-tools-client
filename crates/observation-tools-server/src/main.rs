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
  /// Generate an API key using the secret from OBSERVATION_TOOLS_API_SECRET
  GenerateKey,
}

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

  let cli = Cli::parse();

  match cli.command {
    Commands::Serve { data_dir } => {
      // Read PORT from environment or use default
      let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

      let bind_addr: SocketAddr = ([0, 0, 0, 0], port).into();

      // Read API secret from environment
      let api_secret = std::env::var("OBSERVATION_TOOLS_API_SECRET").ok();

      if api_secret.is_some() {
        tracing::info!("API key authentication enabled");
      } else {
        tracing::warn!("API key authentication disabled - set OBSERVATION_TOOLS_API_SECRET to enable");
      }

      let config = Config::new()
        .with_bind_addr(bind_addr)
        .with_data_dir(data_dir)
        .with_api_secret(api_secret);

      let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
      let server = Server::new(config);
      server.run(listener).await?;
    }
    Commands::GenerateKey => {
      let secret = std::env::var("OBSERVATION_TOOLS_API_SECRET")
        .map_err(|_| anyhow::anyhow!("OBSERVATION_TOOLS_API_SECRET environment variable not set"))?;

      let api_key = observation_tools_server::auth::generate_api_key(&secret)?;
      println!("{}", api_key);
    }
  }

  Ok(())
}
