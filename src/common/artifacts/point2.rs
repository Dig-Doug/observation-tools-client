#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumber;
use crate::artifacts::Geometry2;
use crate::artifacts::Number;
use crate::artifacts::Object2;
use wasm_bindgen::prelude::*;

/// A 2D point.
////#[doc = docify::embed_run!("tests/examples.rs", point2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
////#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Point2 {
    pub x: Number,
    pub y: Number,
}

// Rust only functions
impl Point2 {
    /// Create a point at (x, y).
    pub fn new(x: impl Into<Number>, y: impl Into<Number>) -> Point2 {
        Point2 {
            x: x.into(),
            y: y.into(),
        }
    }
}

// Rust+JS functions
////#[wasm_bindgen]
impl Point2 {
    /// Create a point at (0,0).
    pub fn origin() -> Point2 {
        Point2::new(0.0, 0.0)
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
//#[wasm_bindgen]
impl Point2 {
    #[wasm_bindgen(constructor)]
    pub fn new_js(x: NumberOrNumber, y: NumberOrNumber) -> Result<Point2, crate::anyhow::Error> {
        Ok(Point2::new(
            Number::from_js_value(x)?,
            Number::from_js_value(y)?,
        ))
    }
}

impl Into<Object2> for Point2 {
    fn into(self) -> Object2 {
        Object2::new(Geometry2::Point2(self))
    }
}

impl<A, B> From<(A, B)> for Point2
where
    A: Into<Number>,
    B: Into<Number>,
{
    fn from((x, y): (A, B)) -> Point2 {
        Point2::new(x, y)
    }
}
