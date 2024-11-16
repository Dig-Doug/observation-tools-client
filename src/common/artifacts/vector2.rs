#[cfg(feature = "wasm")]
use crate::artifacts::number::NumberOrNumberBuilder;
#[cfg(feature = "wasm")]
use crate::artifacts::ArtifactError;
use crate::artifacts::Number;
use nalgebra::Scalar;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2 {
    #[wasm_bindgen(skip)]
    pub x: Number,
    #[wasm_bindgen(skip)]
    pub y: Number,
}

#[wasm_bindgen]
impl Vector2 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
    ) -> Result<Vector2, ArtifactError> {
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

impl<T: Scalar + Into<Number>> Into<Vector2> for nalgebra::Vector2<T> {
    fn into(self) -> Vector2 {
        Vector2::new(self.x.clone().into(), self.y.clone().into())
    }
}

impl Into<nalgebra::Vector2<f64>> for Vector2 {
    fn into(self) -> nalgebra::Vector2<f64> {
        nalgebra::Vector2::new(self.x.into(), self.y.into())
    }
}
