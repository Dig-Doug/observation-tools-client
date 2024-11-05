use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use crate::artifacts::Vector2;
use wasm_bindgen::prelude::*;

/// An axis-aligned rectangle.
////#[doc = docify::embed_run!("tests/examples.rs", rect2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Rect2 {
    pub size: Vector2,
}

//#[wasm_bindgen]
impl Rect2 {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(size: Vector2) -> Rect2 {
        Rect2 { size }
    }
}

impl Rect2 {
    pub fn from(size: impl Into<Vector2>) -> Rect2 {
        Rect2::new(size.into())
    }
}

impl Into<Object2> for Rect2 {
    fn into(self) -> Object2 {
        Object2::new(Geometry2::Rect2(self))
    }
}
