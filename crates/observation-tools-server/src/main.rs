//! Observation Tools Server
//!
//! HTTP API backend that collects, stores, indexes, and serves logged
//! observations.

use clap::Parser;
use observation_tools_server::auth::ApiKeySecret;
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
      let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
      let bind_addr: SocketAddr = ([0, 0, 0, 0], port).into();
      let config = Config::new()
        .with_bind_addr(bind_addr)
        .with_data_dir(data_dir)
        .with_api_secret(ApiKeySecret::from_env()?);
      let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
      let server = Server::new(config);
      server.run(listener).await?;
    }
    Commands::GenerateKey => {
      let Some(api_secret) = ApiKeySecret::from_env()? else {
        eprintln!(
          "Error: No api key secret found. Please set the {} environment variable.",
          observation_tools_server::auth::ENV_API_KEY_SECRET
        );
        std::process::exit(1);
      };

      let api_key = observation_tools_server::auth::generate_api_key(&api_secret)?;
      println!("{}", api_key);
    }
  }

  Ok(())
}
