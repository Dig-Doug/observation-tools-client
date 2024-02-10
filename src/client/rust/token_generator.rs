use crate::util::ClientError;
use crate::util::GenericError;
use cached::proc_macro::cached;
use cached::Cached;
use oauth2::basic::BasicErrorResponse;
use oauth2::basic::BasicRevocationErrorResponse;
use oauth2::basic::BasicTokenIntrospectionResponse;
use oauth2::basic::BasicTokenType;
use oauth2::reqwest::async_http_client;
use oauth2::AuthType;
use oauth2::AuthUrl;
use oauth2::AuthorizationCode;
use oauth2::Client;
use oauth2::ClientId;
use oauth2::ClientSecret;
use oauth2::CsrfToken;
use oauth2::DeviceAuthorizationResponse;
use oauth2::DeviceAuthorizationUrl;
use oauth2::ExtraDeviceAuthorizationFields;
use oauth2::ExtraTokenFields;
use oauth2::PkceCodeChallenge;
use oauth2::RedirectUrl;
use oauth2::RevocationUrl;
use oauth2::Scope;
use oauth2::StandardRevocableToken;
use oauth2::StandardTokenResponse;
use oauth2::TokenResponse;
use oauth2::TokenUrl;
use serde::Deserialize;
use serde::Serialize;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::time::Duration;
use std::time::Instant;
use url::Url;

#[derive(Debug, Clone)]
pub enum TokenGenerator {
    /// Generate a URL to complete authentication in a browser.
    OAuth2BrowserFlow,
    /// Generate a code you can use to sign in on another device. Use this flow
    /// when the execution environment doesn't have good input methods.
    OAuth2DeviceCodeFlow,
    #[doc(hidden)]
    /// Use a constant authentication token.
    Constant(String),
}

#[derive(Clone)]
struct GoogleAuthToken {
    token: GoogleTokenResponse,
    request_time: Instant,
}

impl GoogleAuthToken {
    fn expiration_time(&self) -> Instant {
        self.request_time + self.token.expires_in().unwrap_or(Duration::ZERO)
    }

    fn id_token(&self) -> Result<String, GenericError> {
        self.token
            .extra_fields()
            .id_token
            .clone()
            .ok_or(ClientError::GoogleOAuthNoIdToken.into())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoogleTokenFields {
    id_token: Option<String>,
}

impl ExtraTokenFields for GoogleTokenFields {}
impl ExtraDeviceAuthorizationFields for GoogleTokenFields {}

type GoogleTokenResponse = StandardTokenResponse<GoogleTokenFields, BasicTokenType>;
type GoogleClient = Client<
    BasicErrorResponse,
    GoogleTokenResponse,
    BasicTokenType,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
>;

impl TokenGenerator {
    pub async fn token(&self) -> Result<String, ClientError> {
        match self {
            TokenGenerator::Constant(s) => Ok(s.clone()),
            TokenGenerator::OAuth2DeviceCodeFlow => {
                self.device_flow().await.map_err(ClientError::from_string)
            }
            TokenGenerator::OAuth2BrowserFlow => {
                self.pkce_flow().await.map_err(ClientError::from_string)
            }
        }
    }

    async fn pkce_flow(&self) -> Result<String, GenericError> {
        let previous_token = {
            let mut cache = PKCE_FLOW.lock().await;
            cache.cache_get(&()).cloned()
        };

        let token = 'token: {
            if let Some(previous_token) = previous_token.clone() {
                if previous_token.expiration_time() > Instant::now() {
                    break 'token Ok(previous_token);
                }
            }
            pkce_flow(previous_token).await
        }?;

        token.id_token()
    }

    async fn device_flow(&self) -> Result<String, GenericError> {
        let previous_token = {
            let mut cache = DEVICE_FLOW.lock().await;
            cache.cache_get(&()).cloned()
        };

        let token = 'token: {
            if let Some(previous_token) = previous_token.clone() {
                if previous_token.expiration_time() > Instant::now() {
                    break 'token Ok(previous_token);
                }
            }
            device_flow(previous_token).await
        }?;

        token.id_token()
    }
}

fn pkce_flow_client(redirect_address: SocketAddr) -> Result<GoogleClient, GenericError> {
    let google_client_id = ClientId::new(
        "939860080094-ne2k06o95j6j58nsahichvndckv9btok.apps.googleusercontent.com".to_string(),
    );
    let google_client_secret = ClientSecret::new("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".to_string());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())?;
    let client = GoogleClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(format!("http://{}", redirect_address))?)
    .set_revocation_uri(RevocationUrl::new(
        "https://oauth2.googleapis.com/revoke".to_string(),
    )?);
    Ok(client)
}

fn device_code_flow_client() -> Result<GoogleClient, GenericError> {
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())?;
    let device_auth_url =
        DeviceAuthorizationUrl::new("https://oauth2.googleapis.com/device/code".to_string())?;
    Ok(GoogleClient::new(
        ClientId::new(
            "939860080094-gesblh8dc3j1v7num3h7igit60e181ke.apps.googleusercontent.com".to_string(),
        ),
        Some(ClientSecret::new(
            "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".to_string(),
        )),
        auth_url,
        Some(token_url),
    )
    .set_device_authorization_url(device_auth_url)
    .set_auth_type(AuthType::RequestBody))
}

#[cached(
    size = 1,
    key = "()",
    convert = r#"{ () }"#,
    result = true,
    sync_writes = true
)]
async fn pkce_flow(
    previous_token: Option<GoogleAuthToken>,
) -> Result<GoogleAuthToken, GenericError> {
    let listener = TcpListener::bind("localhost:0")?;
    let client = pkce_flow_client(listener.local_addr()?)?;
    if let Some(refresh_token) = previous_token
        .as_ref()
        .and_then(|token| token.token.refresh_token())
    {
        let request_time = Instant::now();
        if let Ok(new_token) = client
            .exchange_refresh_token(refresh_token)
            .request_async(async_http_client)
            .await
        {
            return Ok(GoogleAuthToken {
                token: new_token,
                request_time,
            });
        }
    }

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        r#"
#############################################
############# observation.tools #############
#############################################

Authenticate in your browser: {}

#############################################
#############################################
#############################################
"#,
        authorize_url.to_string()
    );

    let (code, received_csrf) = create_pkce_listen_server(listener).await?;
    if csrf_state.secret() != received_csrf.secret() {
        return Err(ClientError::GoogleOAuthReceivedInvalidState {
            expected: csrf_state.secret().to_string(),
            received: received_csrf.secret().to_string(),
        }
        .into());
    }

    let request_time = Instant::now();
    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(async_http_client)
        .await?;

    Ok(GoogleAuthToken {
        token: token_response,
        request_time,
    })
}

#[cached(
    size = 1,
    key = "()",
    convert = r#"{ () }"#,
    result = true,
    sync_writes = true
)]
async fn device_flow(
    previous_token: Option<GoogleAuthToken>,
) -> Result<GoogleAuthToken, GenericError> {
    let request_time = Instant::now();
    let client = device_code_flow_client()?;
    if let Some(refresh_token) = previous_token
        .as_ref()
        .and_then(|token| token.token.refresh_token())
    {
        if let Ok(new_token) = client
            .exchange_refresh_token(refresh_token)
            .request_async(async_http_client)
            .await
        {
            return Ok(GoogleAuthToken {
                token: new_token,
                request_time,
            });
        }
    }

    let details: DeviceAuthorizationResponse<GoogleTokenFields> = client
        .exchange_device_code()?
        .add_scope(Scope::new("email".to_string()))
        .request_async(async_http_client)
        .await?;

    println!(
        r#"
#############################################
############# observation.tools #############
#############################################

Authenticate in your browser: {}

Enter this code: {}

#############################################
#############################################
#############################################
"#,
        details.verification_uri().to_string(),
        details.user_code().secret().to_string()
    );

    let response = client
        .exchange_device_access_token(&details)
        .request_async(async_http_client, tokio::time::sleep, None)
        .await?;
    Ok(GoogleAuthToken {
        token: response,
        request_time,
    })
}

async fn create_pkce_listen_server(
    listener: TcpListener,
) -> Result<(AuthorizationCode, CsrfToken), GenericError> {
    // TODO(doug): Clean this up
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = include_str!("sign_in_redirect.html");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            return Ok((code, state));
        }
    }

    Err(ClientError::FailedToOpenPKCEServer.into())
}
