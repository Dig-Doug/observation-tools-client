use crate::util::{ClientError, GenericError};
use artifacts_api_rust_proto::Number;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct NumberBuilder {
    pub(crate) proto: Number,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number | NumberBuilder")]
    pub type NumberOrNumberBuilder;
}

#[wasm_bindgen]
impl NumberBuilder {
    pub fn from_js_value(value: NumberOrNumberBuilder) -> Result<NumberBuilder, ClientError> {
        let js_value: &JsValue = value.as_ref();
        if let Some(number) = js_value.as_f64() {
            Ok(number.into())
        } else {
            Ok(NumberBuilder::try_from(js_value).map_err(|e| {
                ClientError::FailedToConvertJsValueToNumber {
                    value: e.to_string(),
                }
            })?)
        }
    }

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
