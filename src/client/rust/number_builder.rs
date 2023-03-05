use artifacts_api_rust_proto::Number;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NumberBuilder {
    pub(crate) proto: Number,
}

#[wasm_bindgen]
impl NumberBuilder {
    pub fn from_f64(d: f64) -> NumberBuilder {
        d.into()
    }
}

impl Into<NumberBuilder> for f64 {
    fn into(self) -> NumberBuilder {
        let mut proto = Number::new();
        proto.d = self;
        NumberBuilder { proto }
    }
}
