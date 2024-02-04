use crate::artifacts::Geometry3Builder;
use crate::artifacts::NumberBuilder;
use crate::artifacts::Object3Builder;
use crate::generated::Sphere;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 3D sphere.
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

impl Into<Geometry3Builder> for SphereBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::sphere(self)
    }
}

impl Into<Object3Builder> for SphereBuilder {
    fn into(self) -> Object3Builder {
        Object3Builder::new(self.into())
    }
}
