use crate::artifacts::Geometry3;
use crate::artifacts::Object3;
use crate::artifacts::Vertex;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 3D mesh
//#[doc = docify::embed_run!("tests/examples.rs", mesh3_example)]
#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mesh {
    #[wasm_bindgen(skip)]
    pub vertices: Vec<Vertex>,
    #[wasm_bindgen(skip)]
    pub indices: Vec<u32>,
}

#[wasm_bindgen]
impl Mesh {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Mesh {
        Mesh {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Mesh {
    pub fn into_object(self) -> Object3 {
        self.into()
    }
}

impl Into<Geometry3> for Mesh {
    fn into(self) -> Geometry3 {
        Geometry3::Mesh(self)
    }
}

impl Into<Object3> for Mesh {
    fn into(self) -> Object3 {
        let builder = Object3::new(self.into());
        builder
    }
}
