use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use custom_error::custom_error;
use protobuf::Message;
use protobuf::well_known_types::Timestamp;
use uuid::Uuid;

pub type GenericError = Box<dyn Error + Send + Sync>;

custom_error! {#[derive(Clone)] pub ClientError
    ServerError{status_code: u16, response: String} = "Server error {status_code}: {response}"
}

pub(crate) fn encode_id_proto(msg: &impl Message) -> String {
  bs58::encode(msg.write_to_bytes().unwrap()).into_string()
}

pub(crate) fn new_uuid_proto() -> artifacts_api_rust_proto::Uuid {
  let uuid = Uuid::new_v4();
  let mut proto = artifacts_api_rust_proto::Uuid::new();
  proto.set_data(uuid.as_bytes().to_vec());
  proto
}

#[cfg(not(feature = "wasm"))]
pub(crate) fn time_now() -> Timestamp {
  let mut t = Timestamp::new();
  let since_the_epoch = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards");
  t.set_seconds(since_the_epoch.as_secs() as i64);
  let nanos = (since_the_epoch - Duration::from_secs(t.seconds as u64)).as_nanos();
  t.set_nanos(nanos as i32);
  t
}

#[cfg(feature = "wasm")]
pub(crate) fn time_now() -> Timestamp {
  let mut t = Timestamp::new();
  let window = web_sys::window().expect("should have a window in this context");
  let performance = window
    .performance()
    .expect("performance should be available");
  let since_the_epoch = perf_to_system(performance.now())
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards");
  t.set_seconds(since_the_epoch.as_secs() as i64);
  let nanos = (since_the_epoch - Duration::from_secs(t.seconds as u64)).as_nanos();
  t.set_nanos(nanos as i32);
  t
}

fn perf_to_system(amt: f64) -> SystemTime {
  let secs = (amt as u64) / 1_000;
  let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
  UNIX_EPOCH + Duration::new(secs, nanos)
}