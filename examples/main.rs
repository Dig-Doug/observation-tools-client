use clap::Parser;
use observation_tools_client::builders::Transform3Builder;
use observation_tools_client::ClientOptions;
use observation_tools_client::TokenGenerator;
use observation_tools_client::UserMetadataBuilder;
use observation_tools_client_examples::generate_stone_wall;
use observation_tools_client_examples::GenericError;
use tracing::info;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    project_id: String,
    #[arg(long)]
    auth_token: String,
    #[arg(long)]
    ui_host: Option<String>,
    #[arg(long)]
    api_host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), GenericError> {
    env_logger::init();

    let args = Args::parse();

    let client = observation_tools_client::Client::new(ClientOptions {
        ui_host: args.ui_host,
        api_host: args.api_host,
        project_id: args.project_id,
        client: None,
        token_generator: TokenGenerator::Constant(args.auth_token),
    })
    .expect("Failed to create client");

    let run_uploader = client
        .create_run(&UserMetadataBuilder::new("examples"))
        .await?;

    let uploader = run_uploader
        .child_uploader(&UserMetadataBuilder::new("generic"))
        .await?;
    // TODO(doug): Should we simplify this to just uploader.child_uploader_3d?
    let uploader_3d = uploader
        .child_uploader_3d(
            &UserMetadataBuilder::new("generate_barn_wall"),
            Transform3Builder::identity(),
        )
        .await?;
    generate_stone_wall(uploader_3d).await?;

    println!("See the output at: {}", run_uploader.viewer_url());

    Ok(())
}
