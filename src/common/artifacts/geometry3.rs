use crate::artifacts::Mesh;
use crate::artifacts::Polygon3;
use crate::artifacts::Sphere;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum Geometry3 {
    Sphere(Sphere),
    Mesh(Mesh),
    Polygon3(Polygon3),
}

#[cfg(feature = "wasm")]
//#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Geometry3 | Sphere | Polygon3 | Mesh")]
    pub type IntoGeometry3;
}
