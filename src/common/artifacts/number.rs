extern crate alloc;

use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[derive(Debug, Clone)]
pub enum Number {
    Double(f64),
}

#[cfg(feature = "wasm")]
//#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number | Number")]
    pub type NumberOrNumberBuilder;
}

impl Number {
    #[cfg(feature = "wasm")]
    pub fn from_js_value(value: NumberOrNumberBuilder) -> Result<Number, crate::anyhow::Error> {
        let js_value: &JsValue = value.as_ref();
        if let Some(number) = js_value.as_f64() {
            Ok(number.into())
        } else {
            Ok(Number::try_from(js_value).map_err(|e| {
                crate::anyhow::Error::FailedToConvertJsValueToNumber {
                    value: e.to_string(),
                }
            })?)
        }
    }

    pub fn from_f64(d: f64) -> Number {
        d.into()
    }
}

impl Into<Number> for f64 {
    fn into(self) -> Number {
        Number::Double(self)
    }
}
