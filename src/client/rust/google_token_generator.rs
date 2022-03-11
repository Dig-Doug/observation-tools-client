use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::ErrorKind;
use google_cloud_auth::{Credential, CredentialConfigBuilder};
use cached::proc_macro::cached;
use log::warn;
use reqwest::Client;

pub type GenericError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
pub struct GoogleTokenGenerator {}

impl GoogleTokenGenerator {
    pub fn new() -> GoogleTokenGenerator {
        GoogleTokenGenerator {}
    }

    pub async fn token(&self, client: Client) -> Result<String, std::io::Error> {
        get_id_token(client).await.map_err(|e| {
            warn!("{}", e);
            std::io::Error::new(ErrorKind::Other, e)
        })
    }
}

#[cached(
time = 300,
result = true,
sync_writes = true,
key = r#"bool"#,
convert = r#"{true}"#
)]
async fn get_id_token(client: Client) -> Result<String, GenericError> {
    let credential = Credential::find_default(
        CredentialConfigBuilder::new()
            .scopes(vec![
                "https://www.googleapis.com/auth/iam".to_string(),
                "https://www.googleapis.com/auth/cloud-platform".to_string(),
            ])
            .build()
            .unwrap(),
    ).await?;

    let mut map = HashMap::new();
    map.insert("audience", "https://api.observation.tools");
    map.insert("include_email", "true");

    // TODO(doug): Figure out how to get from credentials
    let user = env::var("OBS_USER").expect("Failed to get OBS_USER");

    let access_token = credential.access_token().await?.value;
    let mut json: HashMap<String, String> = client
        .post(format!(
            "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateIdToken",
            user
        ))
        .bearer_auth(access_token)
        .json(&map)
        .send()
        .await?
        .json()
        .await?;
    Ok(json.remove("token").unwrap())
}

