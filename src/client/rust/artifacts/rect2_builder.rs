use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Vector2Builder;
use crate::generated::Rect2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// An axis-aligned rectangle.
///
/// # Example
/// ```rust
/// use observation_tools_client::artifacts::Object2Builder;
/// use observation_tools_client::artifacts::Rect2Builder;
/// use observation_tools_client::artifacts::Transform2Builder;
/// use observation_tools_client::artifacts::Vector2Builder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), observation_tools_client::ClientError> {
///     tracing_subscriber::fmt::init();
///     let client = observation_tools_client::test_utils::create_doc_test_client()?;
///
///     // Set up a 2D group:
///     let run = client.create_run("create_rect2")?;
///     let group2d = run.child_uploader_2d("rect2_world")?;
///
///     // Basic usage:
///     group2d.create_object2("my_rect", Rect2Builder::from((10.0, 5.0)))?;
///
///     // Translate the rect, use shorthand (a,b) notation to create vectors and points
///     let mut rect2: Object2Builder = Rect2Builder::from((5.0, 2.5)).into();
///     rect2.add_transform(Transform2Builder::translation((2.5, 5.0)));
///     group2d.create_object2("translated_rect", rect2)?;
///
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Rect2Builder {
    pub(crate) proto: Rect2,
}

#[wasm_bindgen]
impl Rect2Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(size: &Vector2Builder) -> Rect2Builder {
        let mut proto = Rect2::new();
        proto.size = Some(size.proto.clone()).into();
        Rect2Builder { proto }
    }
}

impl Rect2Builder {
    pub fn from(size: impl Into<Vector2Builder>) -> Rect2Builder {
        Rect2Builder::new(&size.into())
    }
}

impl Into<Object2Builder> for Rect2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::rect(self))
    }
}
