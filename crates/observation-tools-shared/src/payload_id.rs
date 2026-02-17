use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for a payload (UUIDv7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9c")]
pub struct PayloadId(Uuid);

impl Serialize for PayloadId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl PayloadId {
  /// Generate a new UUIDv7 payload ID
  pub fn new() -> Self {
    Self(Uuid::now_v7())
  }

  /// Parse from a string
  pub fn parse(s: &str) -> crate::Result<Self> {
    let uuid = Uuid::parse_str(s).map_err(crate::Error::InvalidPayloadId)?;
    Ok(Self(uuid))
  }
}

impl Default for PayloadId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for PayloadId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.as_simple())
  }
}
