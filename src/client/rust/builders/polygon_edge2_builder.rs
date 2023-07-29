use crate::builders::Point2Builder;
use artifacts_api_rust_proto::PolygonEdge2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PolygonEdge2Builder {
    pub(crate) proto: PolygonEdge2,
}

#[wasm_bindgen]
impl PolygonEdge2Builder {
    pub fn from_vertex(vertex: Point2Builder) -> PolygonEdge2Builder {
        let mut proto = PolygonEdge2::new();
        proto.vertex = Some(vertex.proto).into();
        PolygonEdge2Builder { proto }
    }
}

impl PolygonEdge2Builder {
    pub fn new(vertex: impl Into<Point2Builder>) -> PolygonEdge2Builder {
        PolygonEdge2Builder::from_vertex(vertex.into())
    }
}
