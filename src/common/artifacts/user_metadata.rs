use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMetadata {
    #[wasm_bindgen(skip)]
    pub name: String,
    #[wasm_bindgen(skip)]
    pub metadata: HashMap<String, String>,
}

#[cfg(feature = "python")]
#[pyo3::pymethods]
impl UserMetadata {
    #[new]
    pub fn new_py(name: &str) -> Self {
        UserMetadata::new(name)
    }
}

#[wasm_bindgen]
impl UserMetadata {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(name: String) -> Self {
        Self::new_impl(&name)
    }

    pub fn new(name: &str) -> Self {
        Self::new_impl(name)
    }

    pub fn add_metadata_js(&mut self, key: String, value: String) {
        self.add_metadata(key, value);
    }
}

impl UserMetadata {
    pub fn add_metadata<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.metadata.insert(key.into(), value.into());
    }

    fn new_impl(name: &str) -> UserMetadata {
        UserMetadata {
            name: name.to_string(),
            metadata: Default::default(),
        }
    }
}

impl Into<UserMetadata> for &str {
    fn into(self) -> UserMetadata {
        UserMetadata::new(self.into())
    }
}

impl Into<UserMetadata> for String {
    fn into(self) -> UserMetadata {
        UserMetadata::new(&self)
    }
}
