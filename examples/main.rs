use clap::Parser;
use observation_tools_client::ClientError;
use observation_tools_client_examples::run_examples;
use tracing::info;
use tracing_subscriber;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub project_id: String,
    #[arg(long)]
    pub auth_token: String,
    #[arg(long)]
    pub ui_host: Option<String>,
    #[arg(long)]
    pub api_host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    run_examples(
        args.project_id,
        args.auth_token,
        args.ui_host,
        args.api_host,
    )
    .await?;

    Ok(())
}
