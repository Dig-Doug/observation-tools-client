use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

/// Unique identifier for a group
///
/// Group IDs are user-provided strings. By default, a UUID v7 string is generated,
/// but any string value is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "018e9a3a2c1b7e3f8d2a4b5c6d7e8f9b")]
pub struct GroupId(String);

impl GroupId {
  /// Generate a new UUIDv7 group ID
  pub fn new() -> Self {
    Self(uuid::Uuid::now_v7().as_simple().to_string())
  }

  /// Create a nil group ID for placeholder use
  pub fn nil() -> Self {
    Self(String::new())
  }

  /// Parse from a string (accepts any string value)
  pub fn parse(s: &str) -> Self {
    Self(s.to_string())
  }

  /// Create a GroupId from a deterministic u64 value (e.g., tracing span IDs)
  pub fn from_u64(value: u64) -> Self {
    Self(value.to_string())
  }
}

impl Default for GroupId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::fmt::Display for GroupId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<GroupId> for crate::ObservationId {
  fn from(group_id: GroupId) -> Self {
    // If the group ID happens to be a valid UUID, use it directly
    if let Ok(id) = crate::ObservationId::parse(&group_id.0) {
      return id;
    }
    // Otherwise generate a new ObservationId
    crate::ObservationId::new()
  }
}
