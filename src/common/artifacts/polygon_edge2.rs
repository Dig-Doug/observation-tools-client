use crate::artifacts::Point2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon2](crate::artifacts::Polygon2).
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolygonEdge2 {
    #[wasm_bindgen(skip)]
    pub vertex: Point2,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
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
