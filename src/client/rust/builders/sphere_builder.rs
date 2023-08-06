use crate::builders::Geometry3Builder;
use crate::builders::NumberBuilder;
use crate::builders::Object3Builder;
use artifacts_api_rust_proto::Sphere;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
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

impl Into<Object3Builder> for &SphereBuilder {
    fn into(self) -> Object3Builder {
        let mut builder = Object3Builder::new(self.into());
        builder
    }
}
