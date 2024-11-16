use crate::artifact::StructuredData;
use crate::artifacts::text::Text;
use crate::artifacts::ArtifactError;
use crate::artifacts::SeriesPoint;
use pyo3::PyObject;
use pyo3::PyResult;
use pyo3::Python;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 2D object.
#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object1 {
    #[wasm_bindgen(skip)]
    pub data: Object1Data,
    #[wasm_bindgen(skip)]
    pub series_point: Option<SeriesPoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Object1Data {
    Text(Text),
}

#[wasm_bindgen]
impl Object1 {
    pub fn set_series_point(&mut self, series_point: &SeriesPoint) {
        self.series_point = Some(series_point.clone());
    }
}

impl Object1 {
    pub fn new(data: Object1Data) -> Object1 {
        Object1 {
            data,
            series_point: None,
        }
    }
}

#[pyo3::pymethods]
impl Object1 {
    #[new]
    pub fn new_py(py: Python<'_>, data: PyObject) -> PyResult<Object1> {
        if let Ok(data) = data.extract::<Text>(py) {
            Ok(Object1::new(Object1Data::Text(data)))
        } else {
            // TODO(doug): Make this error message more descriptive
            Err(ArtifactError::FailedToCreateObject1)?
        }
    }
}

impl TryInto<StructuredData> for Object1 {
    type Error = ArtifactError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        Ok(StructuredData::Object1(self))
    }
}
