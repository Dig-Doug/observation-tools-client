use crate::builders::Vector2Builder;
use artifacts_api_rust_proto::Transform2;
use artifacts_api_rust_proto::TRS2;
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

impl Transform2Builder {
    pub fn scale<S: Into<Vector2Builder>>(scale: S) -> Transform2Builder {
        let mut trs = TRS2::new();
        trs.scale = Some(scale.into().proto).into();
        let mut proto = Transform2::new();
        *proto.mut_trs() = trs;
        Transform2Builder { proto }
    }
}
