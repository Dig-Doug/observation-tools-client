use serde::Deserialize;
use serde::Serialize;

/// Payload data for an observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
  /// The actual payload data
  pub data: Vec<u8>,

  /// MIME type of the payload (e.g., "text/plain", "application/json")
  pub mime_type: String,

  /// Size of the payload in bytes
  pub size: usize,
}

impl Payload {
  /// Create a new payload from text
  pub fn text(data: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: "text/plain".to_string(),
      size,
    }
  }

  /// Create a new payload from JSON
  pub fn json(data: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: "application/json".to_string(),
      size,
    }
  }

  /// Create a new payload with a custom MIME type
  pub fn with_mime_type(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
    let data = data.into().into_bytes();
    let size = data.len();
    Self {
      data,
      mime_type: mime_type.into(),
      size,
    }
  }

  /// Get data as UTF-8 string (for testing). Panics if not valid UTF-8.
  #[cfg(any(test, feature = "testing"))]
  pub fn data_as_str(&self) -> &str {
    std::str::from_utf8(&self.data).expect("payload data is not valid UTF-8")
  }
}

impl<T> From<T> for Payload
where
  T: Into<String>,
{
  fn from(s: T) -> Self {
    Payload::text(s)
  }
}

/// A wrapper type for markdown content.
///
/// Use this to create observations with markdown payloads that will be
/// rendered as HTML in the UI.
///
/// # Example
/// ```rust
/// use observation_tools_shared::Markdown;
///
/// let md = Markdown::from("# Hello\n\nThis is **bold** text.");
/// ```
#[derive(Debug, Clone)]
pub struct Markdown {
  content: String,
}

impl Markdown {
  /// Create a new Markdown payload from any type that can be converted to a
  /// String.
  pub fn from(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
    }
  }
}

impl From<Markdown> for Payload {
  fn from(md: Markdown) -> Self {
    Payload::with_mime_type(md.content, "text/markdown")
  }
}
