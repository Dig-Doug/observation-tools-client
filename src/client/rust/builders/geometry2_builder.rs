use artifacts_api_rust_proto::{Geometry2, StructuredData};
use wasm_bindgen::prelude::*;
use crate::builders::Point2Builder;

#[wasm_bindgen]
pub struct Geometry2Builder {
    pub(crate) proto: Geometry2,
}

#[wasm_bindgen]
impl Geometry2Builder {
    pub fn point(point: &Point2Builder) -> Geometry2Builder {
        let mut proto = Geometry2::new();
        *proto.mut_point2() = point.proto.clone();
        Geometry2Builder { proto }
    }
}
