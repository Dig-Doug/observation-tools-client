use crate::artifacts::Geometry3;
use crate::artifacts::Object3;
use crate::artifacts::Vertex;
use wasm_bindgen::prelude::*;

/// A 3D mesh
//#[doc = docify::embed_run!("tests/examples.rs", mesh3_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

//#[wasm_bindgen]
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
