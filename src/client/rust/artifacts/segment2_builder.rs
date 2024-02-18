use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Point2Builder;
use crate::generated::Segment2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 2D line segment.
///
/// # Example
/// ```rust
/// use observation_tools_client::artifacts::Object2Builder;
/// use observation_tools_client::artifacts::Point2Builder;
/// use observation_tools_client::artifacts::Rect2Builder;
/// use observation_tools_client::artifacts::Segment2Builder;
/// use observation_tools_client::artifacts::Transform2Builder;
/// use observation_tools_client::artifacts::Vector2Builder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), observation_tools_client::ClientError> {
///     tracing_subscriber::fmt::init();
///     let client = observation_tools_client::test_utils::create_doc_test_client()?;
///
///     // Set up a 2D group:
///     let run = client.create_run("create_segment2")?;
///     let group2d = run.child_uploader_2d("segment2_world")?;
///
///     // Basic usage:
///     group2d.create_object2("my_segment", Segment2Builder::new((2.0, 1.0), (4.0, 2.0)))?;
///
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Segment2Builder {
    pub(crate) proto: Segment2,
}

#[wasm_bindgen]
impl Segment2Builder {
    pub fn from_points(start: Point2Builder, end: Point2Builder) -> Segment2Builder {
        let mut proto = Segment2::new();
        proto.start = Some(start.proto).into();
        proto.end = Some(end.proto).into();
        Segment2Builder { proto }
    }
}

impl Segment2Builder {
    pub fn new(start: impl Into<Point2Builder>, end: impl Into<Point2Builder>) -> Segment2Builder {
        Segment2Builder::from_points(start.into(), end.into())
    }
}

impl Into<Object2Builder> for Segment2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::segment(self))
    }
}
