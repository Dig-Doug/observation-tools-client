use observation_tools_client::Client;
use observation_tools_client::ClientBuilder;
use std::time::Duration;
use tokio::time::sleep;

/// Test server wrapper that provides convenient client creation
pub struct TestServer {
  base_url: String,
  _handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestServer {
  /// Start a new test server on a random port, or use existing server if
  /// OBSERVATION_TOOLS_TEST_SERVER_URL is set
  pub async fn new() -> Self {
    // Check if an external test server URL is provided
    if let Ok(url) = std::env::var("OBSERVATION_TOOLS_TEST_SERVER_URL") {
      return Self {
        base_url: url,
        _handle: None,
      };
    }

    // Otherwise, start a new test server
    let data_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
      .await
      .expect("Failed to bind to random port");

    let addr = listener.local_addr().expect("Failed to get local address");

    let config = observation_tools_server::Config::new()
      .with_bind_addr(addr)
      .with_data_dir(data_dir.path().to_path_buf());

    let server = observation_tools_server::Server::new(config);

    let handle = tokio::spawn(async move {
      // Keep the tempdir alive for the duration of the server
      let _data_dir = data_dir;
      server.run(listener).await.expect("Server failed to run");
    });

    // Give the server a moment to start up
    sleep(Duration::from_millis(300)).await;

    Self {
      base_url: format!("http://{}", addr),
      _handle: Some(handle),
    }
  }

  /// Get the base URL of the test server
  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  /// Create an observation tools client connected to this test server
  pub fn create_client(&self) -> anyhow::Result<Client> {
    Ok(ClientBuilder::new().base_url(&self.base_url).build()?)
  }

  /// Create an OpenAPI client connected to this test server
  pub fn create_api_client(
    &self,
  ) -> anyhow::Result<observation_tools_client::server_client::Client> {
    observation_tools_client::server_client::create_client(&self.base_url)
  }
}
