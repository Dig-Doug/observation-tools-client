use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use crate::artifacts::Point2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 2D line segment.
//#[doc = docify::embed_run!("tests/examples.rs", segment2_example)]
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment2 {
    #[wasm_bindgen(skip)]
    pub start: Point2,
    #[wasm_bindgen(skip)]
    pub end: Point2,
}

#[wasm_bindgen]
impl Segment2 {
    pub fn from_points(start: Point2, end: Point2) -> Segment2 {
        Segment2 { start, end }
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Segment2 {
    pub fn into_object(self) -> Object2 {
        self.into()
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
