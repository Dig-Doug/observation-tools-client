use artifacts_api_rust_proto::{StructuredData, Image2};
use wasm_bindgen::prelude::*;
use crate::builders::Geometry2Builder;

#[wasm_bindgen]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[wasm_bindgen]
impl Image2Builder {
    pub fn new(data: &[u8]) -> Self {
        let mut proto = Image2::new();
        proto.data = data.to_vec();
        Image2Builder { proto }
    }
}

impl Into<StructuredData> for &Image2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_image2() = self.proto.clone();
        s
    }
}

impl Into<Geometry2Builder> for &Image2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::image(self)
    }
}
