use crate::builders::Geometry2Builder;
use artifacts_api_rust_proto::Image2;
use artifacts_api_rust_proto::StructuredData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[wasm_bindgen]
impl Image2Builder {
    pub fn new(data: &[u8], mime_type: &str) -> Self {
        let mut proto = Image2::new();
        proto.data = data.to_vec();
        proto.mime_type = mime_type.to_string();
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
