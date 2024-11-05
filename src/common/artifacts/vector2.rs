#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumber;
use crate::artifacts::Number;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Vector2 {
    pub x: Number,
    pub y: Number,
}

//#[wasm_bindgen]
impl Vector2 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(x: NumberOrNumber, y: NumberOrNumber) -> Result<Vector2, anyhow::Error> {
        Ok(Vector2::from_number_builder(
            Number::from_js_value(x)?,
            Number::from_js_value(y)?,
        ))
    }

    pub fn from_number_builder(x: Number, y: Number) -> Vector2 {
        Vector2 { x, y }
    }
}

impl Vector2 {
    pub fn new(x: impl Into<Number>, y: impl Into<Number>) -> Vector2 {
        Vector2::from_number_builder(x.into(), y.into())
    }
}

impl<A, B> From<(A, B)> for Vector2
where
    A: Into<Number>,
    B: Into<Number>,
{
    fn from((x, y): (A, B)) -> Vector2 {
        Vector2::new(x, y)
    }
}
