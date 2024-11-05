use crate::artifact::ArtifactId;
#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumber;
use crate::artifacts::Number;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SeriesId {
    pub artifact_id: ArtifactId,
}

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Series {
    pub dimensions: Vec<SeriesDimensionData>,
}

//#[wasm_bindgen]
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

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SeriesDimensionId {
    pub artifact_id: ArtifactId,
}

//#[wasm_bindgen]
#[derive(Clone)]
pub struct SeriesDimension {
    pub(crate) proto: SeriesDimensionData,
}

//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SeriesPoint {
    pub series_id: SeriesId,
    pub values: Vec<SeriesDimensionValue>,
}

//#[wasm_bindgen]
impl SeriesPoint {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(
        series_id: &SeriesId,
        dimension_id: &SeriesDimensionId,
        value: NumberOrNumber,
    ) -> Result<SeriesPoint, anyhow::Error> {
        SeriesPoint::new(series_id, dimension_id, Number::from_js_value(value)?)
    }

    #[cfg(feature = "wasm")]
    // TODO(doug): Allow series with multiple dimensions, private intentionally
    // since not implemented
    fn add_dimension_js(
        &mut self,
        dimension: &SeriesDimensionId,
        value: NumberOrNumber,
    ) -> Result<(), anyhow::Error> {
        self.add_dimension(dimension, Number::from_js_value(value)?)
    }
}

impl SeriesPoint {
    pub fn new<N: Into<Number>>(
        series_id: &SeriesId,
        dimension_id: &SeriesDimensionId,
        value: N,
    ) -> Result<SeriesPoint, anyhow::Error> {
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
    ) -> Result<(), anyhow::Error> {
        self.values.push(SeriesDimensionValue {
            dimension_id: dimension.clone(),
            value: value.into(),
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SeriesDimensionData {
    pub id: SeriesDimensionId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SeriesDimensionValue {
    pub dimension_id: SeriesDimensionId,
    pub value: Number,
}
