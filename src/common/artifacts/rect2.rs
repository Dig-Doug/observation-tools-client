use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use crate::artifacts::Vector2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// An axis-aligned rectangle.
////#[doc = docify::embed_run!("tests/examples.rs", rect2_example)]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rect2 {
    #[wasm_bindgen(skip)]
    pub size: Vector2,
}

#[wasm_bindgen]
impl Rect2 {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new(size: Vector2) -> Rect2 {
        Rect2 { size }
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Rect2 {
    pub fn into_object(self) -> Object2 {
        self.into()
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
