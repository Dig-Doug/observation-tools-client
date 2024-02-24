#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::Geometry2Builder;
use crate::artifacts::NumberBuilder;
use crate::artifacts::Object2Builder;
use crate::generated::Point2;
use wasm_bindgen::prelude::*;

/// A 2D point.
#[doc = docify::embed_run!("tests/examples.rs", point2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
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
    ) -> Result<Point2Builder, crate::ClientError> {
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
