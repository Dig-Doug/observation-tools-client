use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

/// Unique identifier for a payload (UUIDv7)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9c")]
pub struct PayloadId(String);

impl PayloadId {
  /// Generate a new UUIDv7 payload ID
  pub fn new() -> Self {
    Self(uuid::Uuid::now_v7().as_simple().to_string())
  }

  /// Get the string value of this payload ID
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl Default for PayloadId {
  fn default() -> Self {
    Self::new()
  }
}
