use observation_tools_client::Client;
use observation_tools_client::ClientBuilder;
use observation_tools_server::auth::{generate_api_key, ApiKeySecret};
use std::time::Duration;
use tokio::time::sleep;

/// Test server wrapper that provides convenient client creation
pub struct TestServer {
  base_url: String,
  api_secret: Option<ApiKeySecret>,
  _handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestServer {
  pub async fn new() -> Self {
    Self::new_internal(None).await
  }

  /// Create a new test server with API key authentication enabled
  #[allow(unused)]
  pub async fn new_with_auth() -> anyhow::Result<Self> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random1: u64 = rng.gen();
    let random2: u64 = rng.gen();
    let secret = format!("test-secret-key-{:016x}{:016x}", random1, random2);
    let api_secret = ApiKeySecret::new(&secret)?;
    Ok(Self::new_internal(Some(api_secret)).await)
  }

  async fn new_internal(api_secret: Option<ApiKeySecret>) -> Self {
    if let Ok(url) = std::env::var("SERVER_URL") {
      return Self {
        base_url: url,
        api_secret,
        _handle: None,
      };
    }

    let data_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
      .await
      .expect("Failed to bind to random port");

    let addr = listener.local_addr().expect("Failed to get local address");

    let config = observation_tools_server::Config::new()
      .with_bind_addr(addr)
      .with_data_dir(data_dir.path().to_path_buf())
      .with_api_secret(api_secret.clone());

    let server = observation_tools_server::Server::new(config);

    let handle = tokio::spawn(async move {
      let _data_dir = data_dir;
      server.run(listener).await.expect("Server failed to run");
    });

    sleep(Duration::from_millis(300)).await;

    Self {
      base_url: format!("http://{}", addr),
      api_secret,
      _handle: Some(handle),
    }
  }

  /// Get the base URL of the test server
  #[allow(unused)]
  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  /// Generate a valid API key for this server
  #[allow(unused)]
  pub fn generate_api_key(&self) -> anyhow::Result<String> {
    let Some(ref secret) = self.api_secret else {
      anyhow::bail!("API secret not configured for this test server");
    };
    Ok(generate_api_key(secret)?)
  }

  /// Create an observation tools client connected to this test server
  #[allow(unused)]
  pub fn create_client(&self) -> anyhow::Result<Client> {
    Ok(ClientBuilder::new().base_url(&self.base_url).build()?)
  }

  /// Create an observation tools client with API key authentication
  #[allow(unused)]
  pub fn create_client_with_api_key(&self, api_key: &str) -> anyhow::Result<Client> {
    Ok(
      ClientBuilder::new()
        .base_url(&self.base_url)
        .api_key(api_key)
        .build()?,
    )
  }

  /// Create an OpenAPI client connected to this test server
  #[allow(unused)]
  pub fn create_api_client(
    &self,
  ) -> anyhow::Result<observation_tools_client::server_client::Client> {
    observation_tools_client::server_client::create_client(&self.base_url, None)
  }

  /// Create an OpenAPI client with API key authentication
  #[allow(unused)]
  pub fn create_api_client_with_api_key(
    &self,
    api_key: &str,
  ) -> anyhow::Result<observation_tools_client::server_client::Client> {
    observation_tools_client::server_client::create_client(
      &self.base_url,
      Some(api_key.to_string()),
    )
  }
}
