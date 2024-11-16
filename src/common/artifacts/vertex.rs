use crate::artifacts::Point3;
use crate::artifacts::Vector3;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vertex {
    #[wasm_bindgen(skip)]
    pub position: Point3,
    #[wasm_bindgen(skip)]
    pub normal: Option<Vector3>,
}

// Rust only functions
impl Vertex {
    pub fn new<P: Into<Point3>>(position: P) -> Vertex {
        Vertex {
            position: position.into(),
            normal: None,
        }
    }
}

// JS only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Vertex {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(position: Point3) -> Vertex {
        Vertex::new(position)
    }
}

// Rust+JS functions
#[wasm_bindgen]
impl Vertex {
    pub fn set_normal(&mut self, normal: Vector3) {
        self.normal = Some(normal);
    }
}
