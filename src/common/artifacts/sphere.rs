use crate::artifacts::Geometry3;
use crate::artifacts::Number;
use crate::artifacts::Object3;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 3D sphere.
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sphere {
    #[wasm_bindgen(skip)]
    pub radius: Number,
}

#[wasm_bindgen]
impl Sphere {
    pub fn from_number_builder(radius: Number) -> Sphere {
        Sphere { radius }
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Sphere {
    pub fn into_object(self) -> Object3 {
        self.into()
    }
}

impl Sphere {
    pub fn new(radius: impl Into<Number>) -> Sphere {
        Sphere::from_number_builder(radius.into())
    }
}

impl Into<Geometry3> for Sphere {
    fn into(self) -> Geometry3 {
        Geometry3::Sphere(self)
    }
}

impl Into<Object3> for Sphere {
    fn into(self) -> Object3 {
        Object3::new(self.into())
    }
}
