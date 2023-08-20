use crate::builders::Geometry2Builder;
use crate::builders::Object2Builder;
use crate::builders::Transform2Builder;
use crate::builders::Vector2Builder;
use artifacts_api_rust_proto::Rect2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Rect2Builder {
    pub(crate) proto: Rect2,
}

#[wasm_bindgen]
impl Rect2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new(size: &Vector2Builder) -> Rect2Builder {
        let mut proto = Rect2::new();
        proto.size = Some(size.proto.clone()).into();
        Rect2Builder { proto }
    }
}

impl Rect2Builder {
    pub fn from(size: impl Into<Vector2Builder>) -> Rect2Builder {
        Rect2Builder::new(&size.into())
    }
}

impl Into<Object2Builder> for Rect2Builder {
    fn into(self) -> Object2Builder {
        let mut builder = Object2Builder::new(Geometry2Builder::rect(&self));
        builder
    }
}