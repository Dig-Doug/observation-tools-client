use crate::artifacts::Geometry2Builder;
#[cfg(feature = "wasm")]
use crate::artifacts::IntoGeometry2Builder;
use crate::artifacts::SeriesPointBuilder;
use crate::artifacts::Transform2Builder;
use crate::ClientError;
use crate::PublicArtifactId;
use observation_tools_common::proto::structured_data;
use observation_tools_common::proto::Object2;
use observation_tools_common::proto::StructuredData;
use wasm_bindgen::prelude::*;

/// A 2D object.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Object2Builder {
    pub(crate) proto: Object2,
    pub(crate) series_point: Option<SeriesPointBuilder>,
}

#[wasm_bindgen]
impl Object2Builder {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(value: IntoGeometry2Builder) -> Result<Object2Builder, ClientError> {
        use crate::artifacts::Image2Builder;
        use crate::artifacts::Point2Builder;
        use crate::artifacts::Polygon2Builder;
        use crate::artifacts::Rect2Builder;
        use crate::artifacts::Segment2Builder;

        let js_value: &JsValue = value.as_ref();
        if let Ok(val) = Point2Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Segment2Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Polygon2Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Image2Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Rect2Builder::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Geometry2Builder::try_from(js_value) {
            return Ok(Object2Builder::new(val));
        }
        Err(ClientError::FailedToCreateGeometry2Builder)
    }

    pub fn add_transform(&mut self, transform: Transform2Builder) {
        self.proto.transforms.push(transform.proto.clone());
    }

    pub fn set_series_point(&mut self, series_point: &SeriesPointBuilder) {
        self.series_point = Some(series_point.clone());
    }
}

impl Object2Builder {
    pub fn new(geometry: Geometry2Builder) -> Object2Builder {
        Object2Builder {
            proto: Object2 {
                geometry: Some(geometry.proto),
                transforms: vec![],
            },
            series_point: None,
        }
    }
}

impl TryInto<StructuredData> for Object2Builder {
    type Error = ClientError;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.proto.transforms.is_empty() {
            return Err(ClientError::NoTransformsInBuilder);
        }
        Ok(StructuredData {
            data: Some(structured_data::Data::Object2(self.proto)),
        })
    }
}

/// Updater for an Object2.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Object2Updater {
    pub(crate) id: PublicArtifactId,
}
