use crate::artifact_uploader_3d::Type3d;
use crate::geometry3_builder::Geometry3Builder;
use crate::number_builder::NumberBuilder;
use crate::transform3_builder::Transform3Builder;
use artifacts_api_rust_proto::{Object3, StructuredData};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Object3Builder {
    pub(crate) proto: Object3,
}

#[wasm_bindgen]
impl Object3Builder {
    #[wasm_bindgen(constructor)]
    pub fn new(geometry: Geometry3Builder) -> Object3Builder {
        let mut proto = Object3::new();
        proto.geometry = Some(geometry.proto).into();
        Object3Builder { proto }
    }

    pub fn add_transform(&mut self, transform: &Transform3Builder) {
        self.proto.transforms.push(transform.proto.clone());
    }
}

impl Into<StructuredData> for &Object3Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_object3() = self.proto.clone();
        s
    }
}

impl Type3d for Object3Builder {
    fn convert_3d_to_raw(&self) -> StructuredData {
        self.into()
    }
}
