use crate::api::new_artifact_id;
use crate::builders::number_builder::NumberOrNumberBuilder;
use crate::builders::NumberBuilder;
use crate::util::ClientError;
use artifacts_api_rust_proto::{
    SeriesData, SeriesDimensionData, SeriesDimensionValue, SeriesPoint,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct PublicSeriesId {
    pub(crate) proto: artifacts_api_rust_proto::SeriesId,
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
        let mut proto = SeriesData::new();
        SeriesBuilder { proto }
    }

    pub fn add_dimension(&mut self, name: &str) -> PublicSeriesDimensionId {
        let mut id = artifacts_api_rust_proto::SeriesDimensionId::new();
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
    pub(crate) proto: artifacts_api_rust_proto::SeriesDimensionId,
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
    pub fn new(
        series_id: &PublicSeriesId,
        dimension_id: &PublicSeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<SeriesPointBuilder, ClientError> {
        let mut proto = SeriesPoint::new();
        proto.series_id = Some(series_id.proto.clone()).into();
        let mut point = SeriesPointBuilder { proto };
        point.add_dimension(dimension_id, value)?;
        Ok(point)
    }

    // TODO(doug): Allow series with multiple dimensions
    fn add_dimension(
        &mut self,
        dimension: &PublicSeriesDimensionId,
        value: NumberOrNumberBuilder,
    ) -> Result<(), ClientError> {
        let mut proto = SeriesDimensionValue::new();
        proto.dimension_id = Some(dimension.proto.clone()).into();
        proto.value = Some(NumberBuilder::from_js_value(value)?.proto).into();
        self.proto.values.push(proto);
        Ok(())
    }
}
