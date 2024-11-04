use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Point2Builder;
use observation_tools_common::proto::Segment2;
use wasm_bindgen::prelude::*;

/// A 2D line segment.
#[doc = docify::embed_run!("tests/examples.rs", segment2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Segment2Builder {
    pub(crate) proto: Segment2,
}

#[wasm_bindgen]
impl Segment2Builder {
    pub fn from_points(start: Point2Builder, end: Point2Builder) -> Segment2Builder {
        Segment2Builder {
            proto: Segment2 {
                start: Some(start.proto),
                end: Some(end.proto),
            },
        }
    }
}

impl Segment2Builder {
    pub fn new(start: impl Into<Point2Builder>, end: impl Into<Point2Builder>) -> Segment2Builder {
        Segment2Builder::from_points(start.into(), end.into())
    }
}

impl Into<Object2Builder> for Segment2Builder {
    fn into(self) -> Object2Builder {
        Object2Builder::new(Geometry2Builder::segment(self))
    }
}
