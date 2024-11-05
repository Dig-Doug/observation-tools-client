mod auth;
mod ingestion;
mod server;

use crate::auth::AuthState;
use crate::ingestion::create_artifact::CreateArtifactState;
use crate::server::ServerState;
use axum::extract::FromRef;
use axum::routing::get;
use axum::Router;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use std::env;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(long)]
    todo: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    match cli.command {
        Commands::Run(args) => {
            let port = env::var("PORT").unwrap_or("8000".to_string());
            let app = Router::new()
                .route("/", get(index))
                .with_state(ServerState {});
            let address = format!("0.0.0.0:{}", port);
            info!("Listening on http://{}", address);
            axum::serve(TcpListener::bind(address).await?, app).await?;
        }
    }

    Ok(())
}

async fn index() -> String {
    "OK".to_string()
}
