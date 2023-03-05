use crate::artifact_uploader_3d::Type3d;
use crate::geometry3_builder::Geometry3Builder;
use crate::number_builder::NumberBuilder;
use artifacts_api_rust_proto::{Sphere, StructuredData};
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

impl Into<StructuredData> for &SphereBuilder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_sphere() = self.proto.clone();
        s
    }
}

impl Type3d for SphereBuilder {
    fn convert_3d_to_raw(&self) -> StructuredData {
        self.into()
    }
}
