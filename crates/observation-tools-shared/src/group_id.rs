use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

/// Unique identifier for a group
///
/// Group IDs are user-provided strings. By default, a UUID v7 string is
/// generated, but any string value is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9b")]
pub struct GroupId(String);

impl GroupId {
  /// Generate a new UUIDv7 group ID
  pub fn new() -> Self {
    Self(uuid::Uuid::now_v7().as_simple().to_string())
  }

  /// Get the string value of this group ID
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl Default for GroupId {
  fn default() -> Self {
    Self::new()
  }
}

impl From<String> for GroupId {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl From<&str> for GroupId {
  fn from(s: &str) -> Self {
    Self(s.to_string())
  }
}
