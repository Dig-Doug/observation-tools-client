#[cfg(feature = "wasm")]
use crate::artifacts::ArtifactError;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Number {
    #[wasm_bindgen(skip)]
    pub data: NumberData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NumberData {
    Double(f64),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number | Number")]
    pub type NumberOrNumberBuilder;
}

impl Number {
    #[cfg(feature = "wasm")]
    pub fn from_js_value(value: NumberOrNumberBuilder) -> Result<Number, ArtifactError> {
        let js_value: &JsValue = value.as_ref();
        if let Some(number) = js_value.as_f64() {
            Ok(number.into())
        } else {
            Ok(Number::try_from(js_value).map_err(|e| {
                ArtifactError::FailedToConvertJsValueToNumber {
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
        Number {
            data: NumberData::Double(self),
        }
    }
}

impl Into<f64> for Number {
    fn into(self) -> f64 {
        match self.data {
            NumberData::Double(d) => d,
        }
    }
}
