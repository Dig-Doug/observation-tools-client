use crate::artifacts::Geometry3Builder;
use crate::artifacts::Object3Builder;
use crate::artifacts::VertexBuilder;
use crate::generated::Mesh;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 3D mesh
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone)]
pub struct MeshBuilder {
    pub(crate) proto: Mesh,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl MeshBuilder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> MeshBuilder {
        let proto = Mesh::new();
        MeshBuilder { proto }
    }

    pub fn add_vertex(&mut self, vertex: VertexBuilder) {
        self.proto.vertices.push(vertex.proto);
    }

    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.proto.indices.push(i0);
        self.proto.indices.push(i1);
        self.proto.indices.push(i2);
    }
}

impl Into<Geometry3Builder> for MeshBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::mesh(self)
    }
}

impl Into<Object3Builder> for MeshBuilder {
    fn into(self) -> Object3Builder {
        let builder = Object3Builder::new(self.into());
        builder
    }
}
