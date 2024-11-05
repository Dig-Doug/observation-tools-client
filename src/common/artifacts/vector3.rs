#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumber;
use crate::artifacts::Number;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: Number,
    pub y: Number,
    pub z: Number,
}

//#[wasm_bindgen]
impl Vector3 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumber,
        y: NumberOrNumber,
        z: NumberOrNumber,
    ) -> Result<Vector3, crate::anyhow::Error> {
        Ok(Vector3::from_number_builder(
            Number::from_js_value(x)?,
            Number::from_js_value(y)?,
            Number::from_js_value(z)?,
        ))
    }

    pub fn from_number_builder(x: Number, y: Number, z: Number) -> Vector3 {
        Vector3 { x, y, z }
    }
}

impl Vector3 {
    pub fn new(x: impl Into<Number>, y: impl Into<Number>, z: impl Into<Number>) -> Vector3 {
        Vector3::from_number_builder(x.into(), y.into(), z.into())
    }
}
