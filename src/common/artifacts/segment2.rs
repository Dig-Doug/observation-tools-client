use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use crate::artifacts::Point2;
use wasm_bindgen::prelude::*;

/// A 2D line segment.
//#[doc = docify::embed_run!("tests/examples.rs", segment2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Segment2 {
    pub start: Point2,
    pub end: Point2,
}

//#[wasm_bindgen]
impl Segment2 {
    pub fn from_points(start: Point2, end: Point2) -> Segment2 {
        Segment2 { start, end }
    }
}

impl Segment2 {
    pub fn new(start: impl Into<Point2>, end: impl Into<Point2>) -> Segment2 {
        Segment2::from_points(start.into(), end.into())
    }
}

impl Into<Object2> for Segment2 {
    fn into(self) -> Object2 {
        Object2::new(Geometry2::Segment2(self))
    }
}
