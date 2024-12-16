use crate::artifacts::Point3;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon3](crate::artifacts::Polygon3).
#[wasm_bindgen]
#[cfg_attr(feature="python", pyo3::pyclass)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolygonEdge3 {
    #[wasm_bindgen(skip)]
    pub vertex: Point3,
}

#[wasm_bindgen]
impl PolygonEdge3 {
    pub fn from_vertex(vertex: Point3) -> PolygonEdge3 {
        PolygonEdge3 { vertex }
    }
}

impl PolygonEdge3 {
    pub fn new(vertex: impl Into<Point3>) -> PolygonEdge3 {
        PolygonEdge3::from_vertex(vertex.into())
    }
}
