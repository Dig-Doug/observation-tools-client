use crate::builders::Transform2Builder;
use crate::builders::{Geometry2Builder, SeriesPointBuilder};
use crate::PublicArtifactId;
use artifacts_api_rust_proto::Object2;
use artifacts_api_rust_proto::StructuredData;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Object2Builder {
    pub(crate) proto: Object2,
    pub(crate) series_point: Option<SeriesPointBuilder>,
}

#[wasm_bindgen]
impl Object2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new(geometry: Geometry2Builder) -> Object2Builder {
        let mut proto = Object2::new();
        proto.geometry = Some(geometry.proto).into();
        Object2Builder {
            proto,
            series_point: None,
        }
    }

    pub fn add_transform(&mut self, transform: &Transform2Builder) {
        self.proto.transforms.push(transform.proto.clone());
    }

    pub fn set_series_point(&mut self, series_point: &SeriesPointBuilder) {
        self.series_point = Some(series_point.clone());
    }
}

impl Into<StructuredData> for &Object2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_object2() = self.proto.clone();
        s
    }
}

#[wasm_bindgen]
pub struct Object2Updater {
    pub(crate) id: PublicArtifactId,
}
