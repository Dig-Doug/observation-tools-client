use crate::artifacts::Geometry3;
use crate::artifacts::Number;
use crate::artifacts::Object3;
use wasm_bindgen::prelude::*;

/// A 3D sphere.
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Sphere {
    pub radius: Number,
}

//#[wasm_bindgen]
impl Sphere {
    pub fn from_number_builder(radius: Number) -> Sphere {
        Sphere { radius }
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
