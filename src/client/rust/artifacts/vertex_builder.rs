use crate::artifacts::Point3Builder;
use crate::artifacts::Vector3Builder;
use observation_tools_common::proto::Vertex;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VertexBuilder {
    pub(crate) proto: Vertex,
}

// Rust only functions
impl VertexBuilder {
    pub fn new<P: Into<Point3Builder>>(position: P) -> VertexBuilder {
        VertexBuilder {
            proto: Vertex {
                position: Some(position.into().proto),
                normal: None,
            },
        }
    }
}

// JS only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VertexBuilder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(position: Point3Builder) -> VertexBuilder {
        VertexBuilder::new(position)
    }
}

// Rust+JS functions
#[wasm_bindgen]
impl VertexBuilder {
    pub fn set_normal(&mut self, normal: Vector3Builder) {
        self.proto.normal = Some(normal.proto).into();
    }
}
