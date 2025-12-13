use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/observation_tools_openapi.rs"));

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
  /// Create observations via multipart form
  ///
  /// This endpoint is not part of the generated OpenAPI client because
  /// progenitor doesn't support multipart request bodies.
  ///
  /// The multipart form contains:
  /// - "observations": JSON array of observation metadata (with empty payload.data)
  /// - "{observation_id}": Binary payload data for each observation
  pub async fn create_observations_multipart(
    &self,
    execution_id: &str,
    observations: Vec<observation_tools_shared::models::Observation>,
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

    // Prepare observations metadata (with payload.data cleared) and collect payloads
    let mut observations_metadata = Vec::with_capacity(observations.len());
    let mut payloads: Vec<(String, Vec<u8>)> = Vec::new();

    for mut obs in observations {
      let obs_id = obs.id.to_string();
      let payload_data = std::mem::take(&mut obs.payload.data);

      if !payload_data.is_empty() {
        payloads.push((obs_id, payload_data));
      }

      observations_metadata.push(obs);
    }

    // Add observations JSON part
    let observations_json = serde_json::to_vec(&observations_metadata)?;
    let observations_part = reqwest::multipart::Part::bytes(observations_json)
      .mime_str("application/json")?;
    form = form.part("observations", observations_part);

    // Add payload parts
    for (obs_id, payload_data) in payloads {
      let part = reqwest::multipart::Part::bytes(payload_data);
      form = form.part(obs_id, part);
    }

    let mut request_builder = self.client.post(&url).multipart(form);

    // Add Authorization header if API key is configured
    if let Some(ref api_key) = self.inner.api_key {
      request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = match request_builder.send().await {
      Ok(resp) => resp,
      Err(e) => {
        log::error!(
          "Failed to send create_observations request: url={}, count={}, error={}",
          url,
          observation_count,
          e
        );
        return Err(anyhow::anyhow!(
          "Failed to send create_observations request to {}: {}",
          url,
          e
        ));
      }
    };

    let status = response.status();
    if !status.is_success() {
      let error_text = response
        .text()
        .await
        .unwrap_or_else(|e| format!("(failed to read error response body: {})", e));
      log::error!(
        "Create observations failed with HTTP error: url={}, status={}, body={}",
        url,
        status,
        error_text
      );
      return Err(anyhow::anyhow!(
        "Create observations failed: HTTP {} from {} - {}",
        status,
        url,
        error_text
      ));
    }

    log::trace!(
      "Observations created successfully: count={}",
      observation_count
    );

    Ok(())
  }
}
