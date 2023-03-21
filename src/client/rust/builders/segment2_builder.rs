use crate::artifact_uploader_2d::Type2d;
use crate::builders::{Geometry2Builder, Point2Builder};
use crate::builders::NumberBuilder;
use artifacts_api_rust_proto::{Segment2, StructuredData};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Segment2Builder {
    pub(crate) proto: Segment2,
}

#[wasm_bindgen]
impl Segment2Builder {
    pub fn from_points(start: Point2Builder, end: Point2Builder) -> Segment2Builder {
        let mut proto = Segment2::new();
        proto.start = Some(start.proto).into();
        proto.end = Some(end.proto).into();
        Segment2Builder { proto }
    }
}

impl Segment2Builder {
    pub fn new(start: impl Into<Point2Builder>, end: impl Into<Point2Builder>) -> Segment2Builder {
        Segment2Builder::from_points(start.into(), end.into())
    }
}

impl Into<Geometry2Builder> for &Segment2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::segment(self)
    }
}
