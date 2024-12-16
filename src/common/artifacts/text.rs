use crate::artifacts::object1::Object1Data;
#[cfg(feature = "wasm")]
use crate::artifacts::ArtifactError;
use crate::artifacts::Object1;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Text {
    #[wasm_bindgen(skip)]
    pub text: String,
    #[wasm_bindgen(skip)]
    pub text_type: TextType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextType {
    Plain,
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Text {
    #[new]
    pub fn py_new(text: String) -> Text {
        text.into()
    }
}

impl Into<Text> for String {
    fn into(self) -> Text {
        Text {
            text: self,
            text_type: TextType::Plain,
        }
    }
}

impl Into<Object1> for Text {
    fn into(self) -> Object1 {
        Object1::new(Object1Data::Text(self))
    }
}
