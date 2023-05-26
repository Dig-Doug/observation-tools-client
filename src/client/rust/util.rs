use custom_error::custom_error;
use protobuf::well_known_types::timestamp::Timestamp;
use protobuf::Message;
use std::error::Error;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;
use wasm_bindgen::JsValue;

pub type GenericError = Box<dyn Error + Send + Sync>;

custom_error! {#[derive(Clone)] pub ClientError
    ServerError{status_code: u16, response: String} = "Server error {status_code}: {response}",
    GenericError{message: String} = "Generic error: {message}",
    FailedToConvertJsValueToNumber{value: String} = "Failed to convert {value} into a number",
    FailedToCreateImage = "Failed to create image",
}

impl Into<JsValue> for ClientError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

pub(crate) fn encode_id_proto(msg: &impl Message) -> String {
    bs58::encode(msg.write_to_bytes().unwrap()).into_string()
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

fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

#[cfg(not(feature = "wasm"))]
fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}
