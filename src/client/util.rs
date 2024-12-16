use chrono::DateTime;
use chrono::Utc;
use core::fmt::Debug;
use observation_tools_common::artifacts::ArtifactError;
use std::error::Error;
use std::time::Duration;
use wasm_bindgen::JsValue;

pub type GenericError = Box<dyn Error + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Server error {status_code}: {response}")]
    ServerError { status_code: u16, response: String },
    #[error("Generic error: {message}")]
    GenericError { message: String },
    #[error("Failed to convert {value} into a number")]
    FailedToConvertJsValueToNumber { value: String },
    #[error("Failed to create image")]
    FailedToCreateImage,
    #[error("Failed to convert type to Geometry2Builder")]
    FailedToCreateGeometry2Builder,
    #[error("Failed to convert type to Geometry3Builder")]
    FailedToCreateGeometry3Builder,
    #[error("IO Error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("No transforms on object: Did you forget to add one to the builder?")]
    NoTransformsInBuilder,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("Did not receive an id_token from Google OAuth. This is likely a bug in the client.")]
    GoogleOAuthNoIdToken,
    #[error(
        "Received an invalid state from Google OAuth. Expected: {expected}, Received: {received}"
    )]
    GoogleOAuthReceivedInvalidState { expected: String, received: String },
    #[error("Failed to open OAuth2 PKCE server")]
    FailedToOpenPKCEServer,
    #[error("Artifact error: {source}")]
    ArtifactError {
        #[from]
        source: ArtifactError,
    },
}

impl ClientError {
    pub(crate) fn from_string(message: impl Debug) -> ClientError {
        ClientError::GenericError {
            message: format!("Error: {:?}", message),
        }
    }
}

impl Into<JsValue> for ClientError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

#[cfg(feature = "python")]
impl From<ClientError> for pyo3::PyErr {
    fn from(err: ClientError) -> pyo3::PyErr {
        use pyo3::exceptions::PyValueError;
        PyValueError::new_err(err.to_string())
    }
}

pub(crate) fn time_now() -> DateTime<Utc> {
    let since_the_epoch = since_epoch();
    let seconds = since_the_epoch.as_secs();
    let nanos = (since_the_epoch - Duration::from_secs(seconds)).as_nanos();
    DateTime::from_timestamp(seconds as i64, nanos as u32).expect("Invalid timestamp")
}

#[cfg(feature = "wasm")]
fn since_epoch() -> Duration {
    Duration::from_millis(js_sys::Date::now() as u64)
}

#[cfg(not(feature = "wasm"))]
fn since_epoch() -> Duration {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}
