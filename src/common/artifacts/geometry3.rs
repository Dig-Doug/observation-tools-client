use crate::artifacts::Mesh;
use crate::artifacts::Polygon3;
use crate::artifacts::Sphere;
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Geometry3 {
    Sphere(Sphere),
    Mesh(Mesh),
    Polygon3(Polygon3),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Geometry3 | Sphere | Polygon3 | Mesh")]
    pub type IntoGeometry3;
}
