use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RunId {
    pub(crate) id: String,
}

#[wasm_bindgen]
impl RunId {
    pub fn to_string(&self) -> String {
        self.id.clone()
    }
}
