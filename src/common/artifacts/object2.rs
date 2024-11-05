use crate::artifact::ArtifactId;
use crate::artifact::StructuredData;
use crate::artifacts::Geometry2;
#[cfg(feature = "wasm")]
use crate::artifacts::IntoGeometry2;
use crate::artifacts::SeriesPoint;
use crate::artifacts::Transform2;
use anyhow::anyhow;
use wasm_bindgen::prelude::*;

/// A 2D object.
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Object2 {
    pub geometry: Geometry2,
    pub transforms: Vec<Transform2>,
    pub(crate) series_point: Option<SeriesPoint>,
}

//#[wasm_bindgen]
impl Object2 {
    #[cfg(feature = "wasm")]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(value: IntoGeometry2) -> Result<Object2, anyhow::Error> {
        use crate::artifacts::Image2;
        use crate::artifacts::Point2;
        use crate::artifacts::Polygon2;
        use crate::artifacts::Rect2;
        use crate::artifacts::Segment2;

        let js_value: &JsValue = value.as_ref();
        if let Ok(val) = Point2::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Segment2::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Polygon2::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Image2::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Rect2::try_from(js_value) {
            return Ok(val.into());
        }
        if let Ok(val) = Geometry2::try_from(js_value) {
            return Ok(Object2::new(val));
        }
        Err(anyhow::Error::FailedToCreateGeometry2)
    }

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
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StructuredData, Self::Error> {
        if self.transforms.is_empty() {
            return Err(anyhow!("No transforms in Object2"));
        }
        Ok(StructuredData::Object2(self))
    }
}

/// Updater for an Object2.
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Object2Updater {
    pub(crate) id: ArtifactId,
}
