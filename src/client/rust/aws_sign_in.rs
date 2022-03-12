use std::collections::HashMap;
use std::convert::Infallible;

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use log::{error, info};
use reqwest::Client;
use tokio::runtime::Runtime;
use url::Url;
use serde::{Deserialize, Serialize};
use cached::proc_macro::cached;

use tokio::sync::oneshot::Receiver;

#[derive(Clone)]
pub struct SignInWithAwsTokenGenerator {
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) client: Client,
}

const CLIENT_ID: &str = "6c3u78agu87ni0nvr1skcqdh83";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AwsToken {
    id_token: String,
    access_token: String,
    refresh_token: String,
    expires_in: i32,
    token_type: String,
}

const COGNITO_HOST: &str = "https://observation-tools.auth.us-east-1.amazoncognito.com";

// https://aws.amazon.com/blogs/mobile/understanding-amazon-cognito-user-pool-oauth-2-0-grants/
impl SignInWithAwsTokenGenerator {
    pub async fn token(&self) -> Result<String, std::io::Error> {
        token(&self.runtime, &self.client).await
    }
}

// TODO(doug): Refresh the token without user input
// TODO(doug): Use the token's expiration, not a hardcoded value
// TODO(doug): Cache the token locally in a file so we don't need to prompt every time
#[cached(
time = 3000,
result = true,
sync_writes = true,
key = r#"bool"#,
convert = r#"{true}"#
)]
async fn token(runtime: &Runtime, client: &Client) -> Result<String, std::io::Error> {
    let port = 59437;
    let redirect_uri = format!("http://localhost:{}", port);

    let (tx, rx) = channel::<String>();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    runtime.spawn(start_server(port, redirect_uri.clone(), tx, shutdown_rx));

    let code = rx.recv_timeout(Duration::from_secs(300));
    let _ = shutdown_tx.send(());
    let code = match code {
        Ok(code) => code,
        Err(_) => {
            panic!("Failed to get authorization code. Did you sign in?")
        }
    };

    let mut map = HashMap::new();
    map.insert("grant_type", "authorization_code");
    map.insert("code", &code);
    map.insert("client_id", CLIENT_ID);
    map.insert("redirect_uri", &redirect_uri);

    let json: AwsToken = client
        .post(format!("{}/oauth2/token", COGNITO_HOST))
        .form(&map)
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();
    Ok(json.id_token)
}

async fn start_server(port: u16, redirect_uri: String, tx: Sender<String>, shutdown_rx: Receiver<()>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc = make_service_fn(move |_conn| {
        let tx = tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |_req| {
                let tx = tx.clone();
                async move {
                    let query_pairs = url::form_urlencoded::parse(_req.uri().query().unwrap_or_default().as_bytes());
                    match query_pairs
                        .into_iter()
                        .find(|(key, _)| key == "code") {
                        None => {
                            Ok::<Response<Body>, Infallible>(Response::<Body>::new("Failed to get auth code, please try again".into()))
                        }
                        Some((_, code)) => {
                            let _ = tx.send(code.to_string()).unwrap();
                            Ok::<Response<Body>, Infallible>(Response::<Body>::new("Received auth code, you can close your browser".into()))
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

    let mut url = Url::parse(&format!("{}/oauth2/authorize", COGNITO_HOST))
        .unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("scope", "email openid profile")
        .append_pair("redirect_url", &redirect_uri);
    println!("\n\n###############\n\nTo sign in, open:\n\n{}\n\n###############\n\n", url);

    if let Err(e) = server.await {
        error!("Error shutting down server : {}", e);
    }
    info!("Server shutdown successful");
}
