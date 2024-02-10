use clap::Parser;
use observation_tools_client::ClientError;
use observation_tools_client_examples::run_examples;
use tracing::info;
use tracing_subscriber;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    /// Project to upload data to
    pub project_id: String,
    #[arg(long)]
    /// Use the device code authentication flow instead of the browser flow
    pub device_code_auth: bool,
    #[arg(long)]
    /// **Internal testing** Point the client to a different observation.tools
    /// UI server.
    pub ui_host: Option<String>,
    #[arg(long)]
    /// **Internal testing** Point the client to a different observation.tools
    /// API server.
    pub api_host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    run_examples(
        args.project_id,
        args.device_code_auth,
        args.ui_host,
        args.api_host,
    )
    .await?;

    Ok(())
}
