use crate::artifacts::Point3Builder;
use crate::artifacts::Vector3Builder;
use crate::generated::Vertex;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VertexBuilder {
    pub(crate) proto: Vertex,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl VertexBuilder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(position: Point3Builder) -> VertexBuilder {
        let mut proto = Vertex::new();
        proto.position = Some(position.proto).into();
        VertexBuilder { proto }
    }

    pub fn set_normal(&mut self, normal: Vector3Builder) {
        self.proto.normal = Some(normal.proto).into();
    }
}
