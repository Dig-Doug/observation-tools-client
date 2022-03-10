use std::collections::HashMap;
use std::error::Error;
use google_cloud_auth::{Credential, CredentialConfigBuilder};
use cached::proc_macro::cached;

#[derive(Clone)]
pub struct GoogleTokenGenerator {
    credential: Credential,
}

impl GoogleTokenGenerator {
    pub fn new() -> GoogleTokenGenerator {
        let credential_future = Credential::find_default(
            CredentialConfigBuilder::new()
                .scopes(vec![
                    "https://www.googleapis.com/auth/iam".to_string(),
                    "https://www.googleapis.com/auth/cloud-platform".to_string(),
                ])
                .build()
                .unwrap(),
        );
        GoogleTokenGenerator {
            credential: futures::executor::block_on(credential_future).unwrap(),
        }
    }

    pub fn token(&self) -> String {
        futures::executor::block_on(get_id_token(&self.credential))
    }
}

#[cached(
time = 300,
sync_writes = true,
key = r#"bool"#,
convert = r#"{true}"#
)]
async fn get_id_token(google_auth: &Credential) -> String {
    let mut map = HashMap::new();
    map.insert("audience", "api.observation.tools");
    map.insert("include_email", "true");

    let client = reqwest::Client::new();
    let access_token = google_auth.access_token().await.unwrap().value;
    let mut json: HashMap<String, String> = client
        .post(format!(
            "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateIdToken",
            "backend-server-prod@observation-tools-1.iam.gserviceaccount.com"
        ))
        .bearer_auth(access_token)
        .json(&map)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    json.remove("token").unwrap()
}

