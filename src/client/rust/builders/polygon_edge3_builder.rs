use crate::builders::Point3Builder;
use artifacts_api_rust_proto::PolygonEdge3;
use wasm_bindgen::prelude::*;

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
