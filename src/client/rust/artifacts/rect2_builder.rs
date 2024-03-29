use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Vector2Builder;
use crate::generated::Rect2;
use wasm_bindgen::prelude::*;

/// An axis-aligned rectangle.
#[doc = docify::embed_run!("tests/examples.rs", rect2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Rect2Builder {
    pub(crate) proto: Rect2,
}

#[wasm_bindgen]
impl Rect2Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
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
        Object2Builder::new(Geometry2Builder::rect(self))
    }
}
