#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::Geometry2Builder;
use crate::artifacts::NumberBuilder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Vector2Builder;
use crate::generated::Point2;
use crate::util::ClientError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 2D point.
///
/// # Example
/// ```rust
/// use observation_tools_client::artifacts::Point2Builder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), observation_tools_client::ClientError> {
///     tracing_subscriber::fmt::init();
///     let client = observation_tools_client::test_utils::create_doc_test_client()?;
///
///     // Set up a 2D group:
///     let run = client.create_run("create_point2")?;
///     let group2d = run.child_uploader_2d("point2_world")?;
///
///     // Basic usage:
///     group2d.create_object2("my_point", Point2Builder::new(1.0, 2.0))?;
///     // Point2s can be created directly from tuples:
///     let tuple_point: Point2Builder = (3.0, 4.0).into();
///     group2d.create_object2("my_tuple_point", tuple_point)?;
///
///     // Convert from an nalgebra point:
///     let nalgebra_point: Point2Builder = nalgebra::Point2::new(5.0, 3.0).into();
///     group2d.create_object2("nalgebra_point", nalgebra_point)?;
///
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Point2Builder {
    pub(crate) proto: Point2,
}

// Rust only functions
impl Point2Builder {
    /// Create a point at (x, y).
    pub fn new(x: impl Into<NumberBuilder>, y: impl Into<NumberBuilder>) -> Point2Builder {
        let mut proto = Point2::new();
        proto.x = Some(x.into().proto).into();
        proto.y = Some(y.into().proto).into();
        Point2Builder { proto }
    }
}

// Rust+JS functions
#[wasm_bindgen]
impl Point2Builder {
    /// Create a point at (0,0).
    pub fn origin() -> Point2Builder {
        Point2Builder::new(0.0, 0.0)
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Point2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
    ) -> Result<Point2Builder, ClientError> {
        Ok(Point2Builder::new(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
        ))
    }
}

impl Into<Object2Builder> for Point2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::point(self))
    }
}

impl<A, B> From<(A, B)> for Point2Builder
where
    A: Into<NumberBuilder>,
    B: Into<NumberBuilder>,
{
    fn from((x, y): (A, B)) -> Point2Builder {
        Point2Builder::new(x, y)
    }
}
