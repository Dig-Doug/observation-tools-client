use crate::server_client::types::GetObservation;
use crate::server_client::types::PayloadOrPointerResponse;
use crate::ObservationWithPayload;
use reqwest::multipart::Part;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/observation_tools_openapi.rs"));

impl PayloadOrPointerResponse {
  pub fn as_str(&self) -> Option<&str> {
    match self {
      PayloadOrPointerResponse::Text(t) => Some(t.as_str()),
      _ => None,
    }
  }

  pub fn as_json(&self) -> Option<&serde_json::Value> {
    match self {
      PayloadOrPointerResponse::Json(j) => Some(j),
      _ => None,
    }
  }
}

pub fn create_client(base_url: &str, api_key: Option<String>) -> anyhow::Result<Client> {
  Ok(Client::new_with_client(
    &base_url,
    reqwest::ClientBuilder::new()
      .connect_timeout(Duration::from_secs(30))
      // Increase timeout for large blob uploads (default is 30s which is too short for large
      // payloads)
      .timeout(Duration::from_secs(300)) // 5 minutes for uploads
      .build()?,
    ObservationToolsServerClientOpts { api_key },
  ))
}

#[derive(Clone, Debug)]
pub struct ObservationToolsServerClientOpts {
  pub api_key: Option<String>,
}

pub async fn pre_hook_async(
  client: &ObservationToolsServerClientOpts,
  req: &mut reqwest::Request,
) -> anyhow::Result<()> {
  if let Some(ref api_key) = client.api_key {
    req
      .headers_mut()
      .insert("authorization", format!("Bearer {}", api_key).parse()?);
  }
  Ok(())
}

// Extension methods for Client
impl Client {
  pub async fn create_observations_multipart(
    &self,
    execution_id: &str,
    observations: Vec<ObservationWithPayload>,
  ) -> anyhow::Result<()> {
    let url = format!("{}/api/exe/{}/obs", self.baseurl, execution_id);
    let observation_count = observations.len();

    log::trace!(
      "Creating observations via multipart: url={}, count={}",
      url,
      observation_count
    );

    // Build multipart form
    let mut form = reqwest::multipart::Form::new();

    let (observations, payloads): (Vec<_>, Vec<_>) = observations
      .into_iter()
      .map(|obs| (obs.observation, obs.payload))
      .unzip();
    let payloads = observations
      .iter()
      .zip(payloads.into_iter())
      .map(|(obs, payload)| (obs.id.to_string(), payload.data))
      .collect::<Vec<_>>();

    let observations_json = serde_json::to_vec(&observations)?;
    let observations_part = Part::bytes(observations_json).mime_str("application/json")?;
    form = form.part("observations", observations_part);
    for (obs_id, payload_data) in payloads {
      let part = Part::bytes(payload_data);
      form = form.part(obs_id, part);
    }

    let mut request_builder = self.client.post(&url).multipart(form);
    if let Some(ref api_key) = self.inner.api_key {
      request_builder = request_builder.bearer_auth(api_key);
    }

    let response = request_builder.send().await?;
    let _response = response.error_for_status()?;
    Ok(())
  }
}
