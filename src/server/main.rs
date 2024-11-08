mod auth;
mod graphql;
mod ingestion;
mod server;
mod storage;

use crate::auth::permission::PermissionStorage;
use crate::auth::AuthState;
use crate::graphql::graphql_handler;
use crate::graphql::graphql_playground;
use crate::server::ServerState;
use crate::storage::artifact::ArtifactStorage;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::SqliteConnection;
use observation_tools_common::create_artifact::CreateArtifactRequest;
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
                .route(
                    CreateArtifactRequest::HTTP_PATH,
                    post(ingestion::create_artifact::create_artifact),
                )
                .route("/graphql", get(graphql_playground).post(graphql_handler))
                .with_state(ServerState {
                    artifact_storage: create_sqlite_storage()?,
                    auth_state: AuthState {},
                    permission_storage: PermissionStorage {},
                });
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

fn create_sqlite_storage() -> Result<ArtifactStorage, anyhow::Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(
        "/home/doug/Development/observation-tools-client/tmp.db",
    );
    let pool = Pool::builder()
        .test_on_check_out(true)
        .max_size(1)
        .build(manager)?;
    let storage = storage::sqlite::SqliteArtifactStorage { pool };
    storage.init()?;
    Ok(ArtifactStorage::Local(storage))
}
