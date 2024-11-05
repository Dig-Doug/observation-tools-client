#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::Number;
use wasm_bindgen::prelude::*;

/// A 3D point.
////#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Point3 {
    pub x: Number,
    pub y: Number,
    pub z: Number,
}

////#[wasm_bindgen]
impl Point3 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Point3Builder, crate::anyhow::Error> {
        Ok(Point3Builder::from_number_builder(
            NumberBuilder::from_js_value(x)?,
            NumberBuilder::from_js_value(y)?,
            NumberBuilder::from_js_value(z)?,
        ))
    }

    pub fn from_numbers(x: Number, y: Number, z: Number) -> Point3 {
        Point3 { x: x, y: y, z: z }
    }
}

impl Point3 {
    pub fn new(x: impl Into<Number>, y: impl Into<Number>, z: impl Into<Number>) -> Point3 {
        Point3::from_numbers(x.into(), y.into(), z.into())
    }
}

impl<A, B, C> From<(A, B, C)> for Point3
where
    A: Into<Number>,
    B: Into<Number>,
    C: Into<Number>,
{
    fn from((x, y, z): (A, B, C)) -> Point3 {
        Point3::new(x, y, z)
    }
}
