use crate::artifact::ArtifactId;
#[cfg(feature = "wasm")]
use crate::artifacts::number::NumberOrNumberBuilder;
use crate::artifacts::ArtifactError;
use crate::artifacts::Number;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeriesId {
    #[wasm_bindgen(skip)]
    pub artifact_id: ArtifactId,
}

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Series {
    #[wasm_bindgen(skip)]
    pub dimensions: Vec<SeriesDimensionData>,
}

#[wasm_bindgen]
impl Series {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Series {
        Series { dimensions: vec![] }
    }

    pub fn add_dimension(&mut self, name: &str) -> SeriesDimensionId {
        let id = SeriesDimensionId {
            artifact_id: ArtifactId::new(),
        };

        self.dimensions.push(SeriesDimensionData {
            name: name.to_string(),
            id: id.clone(),
        });

        id
    }
}

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SeriesDimensionId {
    #[wasm_bindgen(skip)]
    pub artifact_id: ArtifactId,
}

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct SeriesDimension {
    #[wasm_bindgen(skip)]
    pub data: SeriesDimensionData,
}

#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeriesPoint {
    #[wasm_bindgen(skip)]
    pub series_id: SeriesId,
    #[wasm_bindgen(skip)]
    pub values: Vec<SeriesDimensionValue>,
}

#[wasm_bindgen]
impl SeriesPoint {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        series_id: &SeriesId,
        dimension_id: &SeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<SeriesPoint, ArtifactError> {
        SeriesPoint::new(series_id, dimension_id, Number::from_js_value(value)?)
    }

    #[cfg(feature = "wasm")]
    // TODO(doug): Allow series with multiple dimensions, private intentionally
    // since not implemented
    fn add_dimension_js(
        &mut self,
        dimension: &SeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<(), ArtifactError> {
        self.add_dimension(dimension, Number::from_js_value(value)?)
    }
}

impl SeriesPoint {
    pub fn new<N: Into<Number>>(
        series_id: &SeriesId,
        dimension_id: &SeriesDimensionId,
        value: N,
    ) -> Result<SeriesPoint, ArtifactError> {
        let mut point = SeriesPoint {
            series_id: series_id.clone(),
            values: vec![],
        };
        point.add_dimension(dimension_id, value)?;
        Ok(point)
    }

    fn add_dimension<N: Into<Number>>(
        &mut self,
        dimension: &SeriesDimensionId,
        value: N,
    ) -> Result<(), ArtifactError> {
        self.values.push(SeriesDimensionValue {
            dimension_id: dimension.clone(),
            value: value.into(),
        });
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeriesDimensionData {
    pub id: SeriesDimensionId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeriesDimensionValue {
    pub dimension_id: SeriesDimensionId,
    pub value: Number,
}
