use crate::artifacts::Point3;
use crate::artifacts::Vector3;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon3](crate::artifacts::Polygon3).
//#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PolygonEdge3 {
    pub vertex: Point3,
}

//#[wasm_bindgen]
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
