use clap::Parser;
use observation_tools_client::builders::Transform3Builder;
use observation_tools_client::builders::UserMetadataBuilder;
use observation_tools_client::ClientOptions;
use observation_tools_client::TokenGenerator;
use observation_tools_client_examples::generate_stone_wall;
use observation_tools_client_examples::GenericError;
use tracing::info;

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
async fn main() -> Result<(), GenericError> {
    env_logger::init();

    let args = Args::parse();

    Ok(())
}
