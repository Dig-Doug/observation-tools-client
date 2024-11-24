#[macro_use]
extern crate num_derive;

mod auth;
mod graphql;
mod ingestion;
mod server;
mod storage;
mod ui;

use crate::auth::AuthState;
use crate::graphql::graphql_handler;
use crate::graphql::graphql_playground;
use crate::server::ServerState;
use crate::storage::artifact::Storage;
use crate::ui::start_embedded_ui;
use async_graphql::ID;
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
use observation_tools_common::GlobalId;
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
    #[command(subcommand)]
    GlobalId(GlobalIdArgs),
}

#[derive(Args)]
struct RunArgs {
    #[arg(long, default_value = "false")]
    ui: bool,
}

#[derive(Subcommand)]
enum GlobalIdArgs {
    #[command(subcommand)]
    Encode(GlobalId),
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
            let _ui = if args.ui {
                Some(start_embedded_ui(&port)?)
            } else {
                None
            };
            run_server(port).await?;
        }
        Commands::GlobalId(args) => match args {
            GlobalIdArgs::Encode(global_id) => {
                let as_id: ID = global_id.into();
                println!("{}", as_id.0);
            }
        },
    }

    Ok(())
}

async fn index() -> String {
    "OK".to_string()
}

async fn run_server(port: String) -> Result<(), anyhow::Error> {
    let app = Router::new()
        .route("/", get(index))
        .route(
            CreateArtifactRequest::HTTP_PATH,
            post(ingestion::create_artifact::create_artifact),
        )
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(ServerState {
            storage: create_sqlite_storage()?,
            auth_state: AuthState {},
        });
    let address = format!("0.0.0.0:{}", port);
    info!("Listening on http://{}", address);
    axum::serve(TcpListener::bind(address).await?, app).await?;

    Ok(())
}

fn create_sqlite_storage() -> Result<Storage, anyhow::Error> {
    // TODO(doug): Fix path
    let manager = ConnectionManager::<SqliteConnection>::new(
        "/home/doug/Development/observation-tools-client/tmp.db",
    );
    let pool = Pool::builder()
        .test_on_check_out(true)
        .max_size(1)
        .build(manager)?;
    let storage = storage::sqlite::SqliteStorage { pool };
    storage.init()?;
    Ok(Storage::Local(storage))
}
