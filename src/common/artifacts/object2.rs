use crate::artifact::StructuredData;
use crate::artifacts::ArtifactError;
use crate::artifacts::Geometry2;
use crate::artifacts::SeriesPoint;
use crate::artifacts::Transform2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 2D object.
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object2 {
    #[wasm_bindgen(skip)]
    pub geometry: Geometry2,
    #[wasm_bindgen(skip)]
    pub transforms: Vec<Transform2>,
    #[wasm_bindgen(skip)]
    pub series_point: Option<SeriesPoint>,
}

#[wasm_bindgen]
impl Object2 {
    pub fn add_transform(&mut self, transform: Transform2) {
        self.transforms.push(transform);
    }

    pub fn set_series_point(&mut self, series_point: &SeriesPoint) {
        self.series_point = Some(series_point.clone());
    }
}

impl Object2 {
    pub fn new(geometry: Geometry2) -> Object2 {
        Object2 {
            geometry,
            transforms: vec![],
            series_point: None,
        }
    }
}

impl TryInto<StructuredData> for Object2 {
    type Error = ArtifactError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.transforms.is_empty() {
            return Err(ArtifactError::NoTransforms);
        }
        Ok(StructuredData::Object2(self))
    }
}
