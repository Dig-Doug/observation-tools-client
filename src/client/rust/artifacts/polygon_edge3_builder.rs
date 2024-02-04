use crate::artifacts::Point3Builder;
use crate::generated::PolygonEdge3;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon3Builder](crate::artifacts::Polygon3Builder).
#[wasm_bindgen]
#[derive(Clone)]
pub struct PolygonEdge3Builder {
    pub(crate) proto: PolygonEdge3,
}

#[wasm_bindgen]
impl PolygonEdge3Builder {
    pub fn from_vertex(vertex: Point3Builder) -> PolygonEdge3Builder {
        let mut proto = PolygonEdge3::new();
        proto.vertex = Some(vertex.proto).into();
        PolygonEdge3Builder { proto }
    }
}

impl PolygonEdge3Builder {
    pub fn new(vertex: impl Into<Point3Builder>) -> PolygonEdge3Builder {
        PolygonEdge3Builder::from_vertex(vertex.into())
    }
}
