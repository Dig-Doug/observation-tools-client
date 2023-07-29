use crate::builders::{MeshBuilder, Polygon3Builder};
use crate::builders::{SphereBuilder};
use artifacts_api_rust_proto::Geometry3;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Geometry3Builder {
    pub(crate) proto: Geometry3,
}

#[wasm_bindgen]
impl Geometry3Builder {
    pub fn sphere(sphere: &SphereBuilder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_sphere() = sphere.proto.clone();
        Geometry3Builder { proto }
    }

    pub fn polygon(polygon: &Polygon3Builder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_polygon() = polygon.proto.clone();
        Geometry3Builder { proto }
    }

    pub fn mesh(mesh: &MeshBuilder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_mesh() = mesh.proto.clone();
        Geometry3Builder { proto }
    }
}
