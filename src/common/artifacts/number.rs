#[cfg(feature = "wasm")]
use crate::artifacts::ArtifactError;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::hash::Hash;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Number {
    #[wasm_bindgen(skip)]
    pub data: NumberData,
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
        self.data.into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NumberData {
    Double(f64),
}

impl Into<f64> for NumberData {
    fn into(self) -> f64 {
        match self {
            NumberData::Double(d) => d,
        }
    }
}

impl Eq for NumberData {}

impl PartialOrd for NumberData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumberData {
    fn cmp(&self, other: &Self) -> Ordering {
        let a: f64 = self.clone().into();
        let b: f64 = other.clone().into();
        a.total_cmp(&b)
    }
}

impl Hash for NumberData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            NumberData::Double(d) => d.to_bits().hash(state),
        }
    }
}
