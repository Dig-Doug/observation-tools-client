use crate::generated::Transform3;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Transform3Builder {
    pub(crate) proto: Transform3,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Transform3Builder {
    pub fn identity() -> Transform3Builder {
        let mut proto = Transform3::new();
        proto.set_identity(true);
        Transform3Builder { proto }
    }
}
