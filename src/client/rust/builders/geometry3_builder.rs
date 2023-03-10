use artifacts_api_rust_proto::{Geometry3, StructuredData};
use wasm_bindgen::prelude::*;
use crate::builders::SphereBuilder;

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
}
