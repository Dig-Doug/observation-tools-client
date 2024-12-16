use crate::artifact::StructuredData;
use crate::artifacts::ArtifactError;
use crate::artifacts::Geometry3;
use crate::artifacts::Transform3;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object3 {
    #[wasm_bindgen(skip)]
    pub geometry: Geometry3,
    #[wasm_bindgen(skip)]
    pub transforms: Vec<Transform3>,
}

#[wasm_bindgen]
impl Object3 {
    pub fn add_transform(&mut self, transform: Transform3) {
        self.transforms.push(transform);
    }
}

impl Object3 {
    pub fn new(geometry: Geometry3) -> Object3 {
        Object3 {
            geometry,
            transforms: vec![],
        }
    }
}

impl TryInto<StructuredData> for Object3 {
    type Error = ArtifactError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.transforms.is_empty() {
            return Err(ArtifactError::NoTransforms);
        }
        Ok(StructuredData::Object3(self))
    }
}
