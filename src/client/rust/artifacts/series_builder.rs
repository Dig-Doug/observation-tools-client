use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use crate::generated::SeriesData;
use crate::generated::SeriesDimensionData;
use crate::generated::SeriesDimensionValue;
use crate::generated::SeriesPoint;
use crate::util::new_artifact_id;
use crate::util::ClientError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PublicSeriesId {
    pub(crate) proto: crate::generated::SeriesId,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SeriesBuilder {
    pub(crate) proto: SeriesData,
}

#[wasm_bindgen]
impl SeriesBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SeriesBuilder {
        let proto = SeriesData::new();
        SeriesBuilder { proto }
    }

    pub fn add_dimension(&mut self, name: &str) -> PublicSeriesDimensionId {
        let mut id = crate::generated::SeriesDimensionId::new();
        id.artifact_id = Some(new_artifact_id()).into();

        let mut proto = SeriesDimensionData::new();
        proto.name = name.to_string();
        proto.id = Some(id.clone()).into();
        self.proto.dimensions.push(proto);

        PublicSeriesDimensionId { proto: id }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PublicSeriesDimensionId {
    pub(crate) proto: crate::generated::SeriesDimensionId,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SeriesDimensionBuilder {
    pub(crate) proto: SeriesDimensionData,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SeriesPointBuilder {
    pub(crate) proto: SeriesPoint,
}

#[wasm_bindgen]
impl SeriesPointBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        series_id: &PublicSeriesId,
        dimension_id: &PublicSeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<SeriesPointBuilder, ClientError> {
        SeriesPointBuilder::new(
            series_id,
            dimension_id,
            NumberBuilder::from_js_value(value)?,
        )
    }

    // TODO(doug): Allow series with multiple dimensions, private intentionally
    // since not implemented
    fn add_dimension_js(
        &mut self,
        dimension: &PublicSeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<(), ClientError> {
        self.add_dimension(dimension, NumberBuilder::from_js_value(value)?)
    }
}

impl SeriesPointBuilder {
    pub fn new<N: Into<NumberBuilder>>(
        series_id: &PublicSeriesId,
        dimension_id: &PublicSeriesDimensionId,
        value: N,
    ) -> Result<SeriesPointBuilder, ClientError> {
        let mut proto = SeriesPoint::new();
        proto.series_id = Some(series_id.proto.clone()).into();
        let mut point = SeriesPointBuilder { proto };
        point.add_dimension(dimension_id, value)?;
        Ok(point)
    }

    fn add_dimension<N: Into<NumberBuilder>>(
        &mut self,
        dimension: &PublicSeriesDimensionId,
        value: N,
    ) -> Result<(), ClientError> {
        let mut proto = SeriesDimensionValue::new();
        proto.dimension_id = Some(dimension.proto.clone()).into();
        proto.value = Some(value.into().proto).into();
        self.proto.values.push(proto);
        Ok(())
    }
}
