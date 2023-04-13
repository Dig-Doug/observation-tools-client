use crate::builders::Geometry3Builder;
use crate::builders::VertexBuilder;
use artifacts_api_rust_proto::Mesh;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MeshBuilder {
    pub(crate) proto: Mesh,
}

#[wasm_bindgen]
impl MeshBuilder {
    #[wasm_bindgen(constructor)]
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

impl Into<Geometry3Builder> for &MeshBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::mesh(self)
    }
}
