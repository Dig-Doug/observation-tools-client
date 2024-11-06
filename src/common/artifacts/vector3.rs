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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector3 {
    #[wasm_bindgen(skip)]
    pub x: Number,
    #[wasm_bindgen(skip)]
    pub y: Number,
    #[wasm_bindgen(skip)]
    pub z: Number,
}

#[wasm_bindgen]
impl Vector3 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Vector3, ArtifactError> {
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

impl<T: Scalar + Into<Number>> Into<Vector3> for nalgebra::Vector3<T> {
    fn into(self) -> Vector3 {
        Vector3::new(
            self.x.clone().into(),
            self.y.clone().into(),
            self.z.clone().into(),
        )
    }
}

impl Into<nalgebra::Vector3<f64>> for Vector3 {
    fn into(self) -> nalgebra::Vector3<f64> {
        nalgebra::Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}
