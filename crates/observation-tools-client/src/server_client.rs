use std::env;
use std::error::Error as StdError;
use std::fmt::Debug;
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
  /// Upload a blob for an observation
  ///
  /// This endpoint is not part of the generated OpenAPI client because
  /// progenitor doesn't support binary request bodies.
  pub async fn upload_observation_blob(
    &self,
    execution_id: &str,
    observation_id: &str,
    data: Vec<u8>,
  ) -> anyhow::Result<()> {
    let url = format!(
      "{}/api/exe/{}/obs/{}/blob",
      self.baseurl, execution_id, observation_id
    );
    let data_size = data.len();

    log::trace!(
      "Uploading blob: url={}, observation_id={}, size={} bytes",
      url,
      observation_id,
      data_size
    );

    let mut request_builder = self
      .client
      .post(&url)
      .header("Content-Type", "application/octet-stream");

    // Add Authorization header if API key is configured
    if let Some(ref api_key) = self.inner.api_key {
      request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = match request_builder.body(data).send().await {
      Ok(resp) => resp,
      Err(e) => {
        // Try to determine the specific error type for better diagnostics
        let error_type = if e.is_timeout() {
          "timeout"
        } else if e.is_connect() {
          "connection failed"
        } else if e.is_request() {
          "request error"
        } else if e.is_body() {
          "body error"
        } else if e.is_decode() {
          "decode error"
        } else {
          "unknown error"
        };

        let mut error_details = format!("{}", e);

        // Add source chain for more context
        if let Some(source) = e.source() {
          error_details.push_str(&format!(" | caused by: {}", source));
          let mut current_source = source;
          while let Some(next_source) = current_source.source() {
            error_details.push_str(&format!(" | {}", next_source));
            current_source = next_source;
          }
        }

        log::error!(
          "Failed to send blob upload request: url={}, observation_id={}, size={} bytes, error_type={}, error={}",
          url,
          observation_id,
          data_size,
          error_type,
          error_details
        );

        return Err(anyhow::anyhow!(
          "Failed to send blob upload request to {} ({}): {} (observation_id={}, size={} bytes)",
          url,
          error_type,
          error_details,
          observation_id,
          data_size
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
        "Blob upload failed with HTTP error: url={}, observation_id={}, status={}, body={}",
        url,
        observation_id,
        status,
        error_text
      );
      return Err(anyhow::anyhow!(
        "Blob upload failed: HTTP {} from {} - {} (observation_id={}, size={} bytes)",
        status,
        url,
        error_text,
        observation_id,
        data_size
      ));
    }

    log::trace!(
      "Blob uploaded successfully: observation_id={}, size={} bytes",
      observation_id,
      data_size
    );

    Ok(())
  }
}
