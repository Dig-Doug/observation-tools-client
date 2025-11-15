use std::env;
use std::fmt::Debug;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/observation_tools_openapi.rs"));

pub fn create_client() -> anyhow::Result<Client> {
  let base_url = env::var("EMBEDDINGS_URL").unwrap_or("http://127.0.0.1:8080".to_string());
  Ok(Client::new_with_client(
    &base_url,
    reqwest::ClientBuilder::new()
      .connect_timeout(Duration::from_secs(30))
      .build()?,
    ObservationToolsServerClientOpts {},
  ))
}

#[derive(Clone, Debug)]
pub struct ObservationToolsServerClientOpts {}

pub async fn pre_hook_async(
  _client: &ObservationToolsServerClientOpts,
  _req: &mut reqwest::Request,
) -> anyhow::Result<()> {
  Ok(())
}
