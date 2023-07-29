use crate::builders::Geometry3Builder;
use crate::builders::NumberBuilder;
use artifacts_api_rust_proto::Sphere;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SphereBuilder {
    pub(crate) proto: Sphere,
}

#[wasm_bindgen]
impl SphereBuilder {
    pub fn from_number_builder(radius: NumberBuilder) -> SphereBuilder {
        let mut proto = Sphere::new();
        proto.radius = Some(radius.proto).into();
        SphereBuilder { proto }
    }
}

impl SphereBuilder {
    pub fn new(radius: impl Into<NumberBuilder>) -> SphereBuilder {
        SphereBuilder::from_number_builder(radius.into())
    }
}

impl Into<Geometry3Builder> for &SphereBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::sphere(self)
    }
}
