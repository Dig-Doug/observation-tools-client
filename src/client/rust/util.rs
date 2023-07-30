use artifacts_api_rust_proto::ArtifactId;
use core::fmt::Debug;
use protobuf::well_known_types::timestamp::Timestamp;
use protobuf::Message;
use std::error::Error;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;
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
    #[error("IO Error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

impl ClientError {
    pub fn from_string(message: impl Debug) -> ClientError {
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

pub(crate) fn new_artifact_id() -> ArtifactId {
    let mut id = ArtifactId::new();
    id.uuid = Some(new_uuid_proto()).into();
    id
}

pub(crate) fn encode_id_proto(msg: &impl Message) -> String {
    bs58::encode(msg.write_to_bytes().unwrap()).into_string()
}

pub(crate) fn decode_id_proto<M: Message>(encoded: &str) -> Result<M, GenericError> {
    let proto_bytes = bs58::decode(encoded).into_vec()?;
    Ok(M::parse_from_bytes(&proto_bytes)?)
}

pub(crate) fn new_uuid_proto() -> artifacts_api_rust_proto::Uuid {
    let uuid = Uuid::new_v4();
    let mut proto = artifacts_api_rust_proto::Uuid::new();
    proto.data = uuid.as_bytes().to_vec();
    proto
}

pub(crate) fn time_now() -> Timestamp {
    let since_the_epoch = since_epoch();
    let mut t = Timestamp::new();
    t.seconds = since_the_epoch.as_secs() as i64;
    let nanos = (since_the_epoch - Duration::from_secs(t.seconds as u64)).as_nanos();
    t.nanos = nanos as i32;
    t
}

#[cfg(feature = "wasm")]
fn since_epoch() -> Duration {
    Duration::from_millis(js_sys::Date::now() as u64)
}

#[cfg(not(feature = "wasm"))]
fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}
