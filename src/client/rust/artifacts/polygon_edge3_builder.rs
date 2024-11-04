use crate::artifacts::Point3Builder;
use observation_tools_common::proto::PolygonEdge3;
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
        PolygonEdge3Builder {
            proto: PolygonEdge3 {
                vertex: Some(vertex.proto),
            },
        }
    }
}

impl PolygonEdge3Builder {
    pub fn new(vertex: impl Into<Point3Builder>) -> PolygonEdge3Builder {
        PolygonEdge3Builder::from_vertex(vertex.into())
    }
}
