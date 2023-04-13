use crate::artifact_uploader_2d::Type2d;
use crate::builders::Geometry2Builder;
use crate::builders::Transform2Builder;
use artifacts_api_rust_proto::Object2;
use artifacts_api_rust_proto::StructuredData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Object2Builder {
    pub(crate) proto: Object2,
}

#[wasm_bindgen]
impl Object2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new(geometry: Geometry2Builder) -> Object2Builder {
        let mut proto = Object2::new();
        proto.geometry = Some(geometry.proto).into();
        Object2Builder { proto }
    }

    pub fn add_transform(&mut self, transform: &Transform2Builder) {
        self.proto.transforms.push(transform.proto.clone());
    }
}

impl Into<StructuredData> for &Object2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_object2() = self.proto.clone();
        s
    }
}

impl Type2d for Object2Builder {
    fn convert_2d_to_raw(&self) -> StructuredData {
        self.into()
    }
}
