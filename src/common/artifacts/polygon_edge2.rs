use crate::artifacts::Point2;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon2](crate::artifacts::Polygon2).
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PolygonEdge2 {
    pub vertex: Point2,
}

#[cfg(feature = "wasm")]
//#[wasm_bindgen]
impl PolygonEdge2 {
    pub fn new_js(vertex: Point2) -> PolygonEdge2 {
        PolygonEdge2::new(vertex)
    }
}

impl PolygonEdge2 {
    pub fn new(vertex: impl Into<Point2>) -> PolygonEdge2 {
        PolygonEdge2 {
            vertex: vertex.into(),
        }
    }
}
