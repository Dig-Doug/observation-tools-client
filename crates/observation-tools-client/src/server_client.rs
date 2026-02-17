use crate::server_client::types::PayloadOrPointerResponse;
use observation_tools_shared::Observation;
use reqwest::multipart::Part;
use serde::Serialize;
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

impl types::GetObservation {
  /// Get the first (default) payload's data, for backward-compatible tests
  pub fn payload(&self) -> &PayloadOrPointerResponse {
    &self.payloads[0].data
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

/// Default payload name used for the primary observation payload
pub const DEFAULT_PAYLOAD_NAME: &str = "default";

/// Entry in the payload manifest sent alongside the multipart form
#[derive(Serialize)]
struct PayloadManifestEntry {
  observation_id: String,
  payload_id: String,
  name: String,
  mime_type: String,
  size: usize,
}

// Extension methods for Client
impl Client {
  pub(crate) async fn create_observations_multipart(
    &self,
    execution_id: &str,
    observations: Vec<Observation>,
    payloads: Vec<crate::client::PayloadUploadData>,
  ) -> anyhow::Result<()> {
    if observations.is_empty() && payloads.is_empty() {
      return Ok(());
    }

    let url = format!("{}/api/exe/{}/obs", self.baseurl, execution_id);

    log::trace!(
      "Creating observations via multipart: url={}, observations={}, payloads={}",
      url,
      observations.len(),
      payloads.len()
    );

    // Build multipart form
    let mut form = reqwest::multipart::Form::new();

    // Part 1: observations JSON
    let observations_json = serde_json::to_vec(&observations)?;
    let observations_part = Part::bytes(observations_json).mime_str("application/json")?;
    form = form.part("observations", observations_part);

    // Part 2: payload manifest JSON
    let manifest: Vec<PayloadManifestEntry> = payloads
      .iter()
      .map(|p| PayloadManifestEntry {
        observation_id: p.observation_id.to_string(),
        payload_id: p.payload_id.to_string(),
        name: p.name.clone(),
        mime_type: p.mime_type.clone(),
        size: p.size,
      })
      .collect();
    let manifest_json = serde_json::to_vec(&manifest)?;
    let manifest_part = Part::bytes(manifest_json).mime_str("application/json")?;
    form = form.part("payload_manifest", manifest_part);

    // Part 3: payload data parts
    for p in payloads {
      let part_key = format!("{}:{}:{}", p.observation_id, p.payload_id, p.name);
      let part = Part::bytes(p.data);
      form = form.part(part_key, part);
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
