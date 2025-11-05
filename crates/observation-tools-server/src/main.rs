//! Observation Tools Server
//!
//! HTTP API backend that collects, stores, indexes, and serves logged observations.

use clap::Parser;
use observation_tools_server::{Config, Server};
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

            let config = Config::new()
                .with_bind_addr(bind_addr)
                .with_data_dir(data_dir);

            let server = Server::new(config);
            server.run().await?;
        }
    }

    Ok(())
}
