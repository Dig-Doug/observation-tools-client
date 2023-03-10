use artifacts_api_rust_proto::{StructuredData, Transform2};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Transform2Builder {
    pub(crate) proto: Transform2,
}

#[wasm_bindgen]
impl Transform2Builder {
    pub fn identity() -> Transform2Builder {
        let mut proto = Transform2::new();
        proto.set_identity(true);
        Transform2Builder { proto }
    }
}
