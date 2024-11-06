#[cfg(feature = "wasm")]
use crate::artifacts::number::NumberOrNumberBuilder;
use crate::artifacts::Number;
use nalgebra::Scalar;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 3D point.
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point3 {
    #[wasm_bindgen(skip)]
    pub x: Number,
    #[wasm_bindgen(skip)]
    pub y: Number,
    #[wasm_bindgen(skip)]
    pub z: Number,
}

#[wasm_bindgen]
impl Point3 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        x: NumberOrNumberBuilder,
        y: NumberOrNumberBuilder,
        z: NumberOrNumberBuilder,
    ) -> Result<Point3, crate::artifacts::ArtifactError> {
        Ok(Point3::from_numbers(
            Number::from_js_value(x)?,
            Number::from_js_value(y)?,
            Number::from_js_value(z)?,
        ))
    }

    pub fn from_numbers(x: Number, y: Number, z: Number) -> Point3 {
        Point3 { x, y, z }
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

impl<T: Scalar + Into<Number>> Into<Point3> for nalgebra::Point3<T> {
    fn into(self) -> Point3 {
        Point3::new(
            self.x.clone().into(),
            self.y.clone().into(),
            self.z.clone().into(),
        )
    }
}

impl Into<nalgebra::Point3<f64>> for Point3 {
    fn into(self) -> nalgebra::Point3<f64> {
        nalgebra::Point3::new(self.x.into(), self.y.into(), self.z.into())
    }
}
