use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone)]
pub struct RunId {
    pub(crate) id: String,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl RunId {
    pub fn to_string(&self) -> String {
        self.id.clone()
    }
}
