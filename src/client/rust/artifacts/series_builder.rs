#[cfg(feature = "wasm")]
use crate::artifacts::number_builder::NumberOrNumberBuilder;
use crate::artifacts::NumberBuilder;
use crate::util::new_artifact_id;
use crate::util::ClientError;
use observation_tools_common::proto::ArtifactType::Series;
use observation_tools_common::proto::SeriesData;
use observation_tools_common::proto::SeriesDimensionData;
use observation_tools_common::proto::SeriesDimensionValue;
use observation_tools_common::proto::SeriesPoint;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PublicSeriesId {
    pub(crate) proto: observation_tools_common::proto::SeriesId,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SeriesBuilder {
    pub(crate) proto: SeriesData,
}

#[wasm_bindgen]
impl SeriesBuilder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> SeriesBuilder {
        SeriesBuilder {
            proto: SeriesData::default(),
        }
    }

    pub fn add_dimension(&mut self, name: &str) -> PublicSeriesDimensionId {
        let id = observation_tools_common::proto::SeriesDimensionId {
            artifact_id: Some(new_artifact_id()),
        };

        self.proto.dimensions.push(SeriesDimensionData {
            name: name.to_string(),
            id: Some(id.clone()),
        });

        PublicSeriesDimensionId { proto: id }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PublicSeriesDimensionId {
    pub(crate) proto: observation_tools_common::proto::SeriesDimensionId,
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
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
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

    #[cfg(feature = "wasm")]
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
        let mut point = SeriesPointBuilder {
            proto: SeriesPoint {
                series_id: Some(series_id.proto.clone()),
                values: vec![],
            },
        };
        point.add_dimension(dimension_id, value)?;
        Ok(point)
    }

    fn add_dimension<N: Into<NumberBuilder>>(
        &mut self,
        dimension: &PublicSeriesDimensionId,
        value: N,
    ) -> Result<(), ClientError> {
        self.proto.values.push(SeriesDimensionValue {
            dimension_id: Some(dimension.proto.clone()),
            value: Some(value.into().proto),
        });
        Ok(())
    }
}
