use crate::builders::Geometry2Builder;
use crate::builders::Object2Builder;
use crate::builders::Point2Builder;
use artifacts_api_rust_proto::Segment2;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
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

impl Into<Object2Builder> for &Segment2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::segment(self))
    }
}
