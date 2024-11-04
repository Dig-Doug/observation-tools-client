use observation_tools_common::proto::transform3;
use observation_tools_common::proto::Transform3;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Transform3Builder {
    pub(crate) proto: Transform3,
}

#[wasm_bindgen]
impl Transform3Builder {
    pub fn identity() -> Transform3Builder {
        Transform3Builder {
            proto: Transform3 {
                data: Some(transform3::Data::Identity(true)),
            },
        }
    }
}
