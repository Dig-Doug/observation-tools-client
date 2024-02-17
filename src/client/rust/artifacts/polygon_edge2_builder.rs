use crate::artifacts::Point2Builder;
use crate::generated::PolygonEdge2;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon2Builder](crate::artifacts::Polygon2Builder).
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct PolygonEdge2Builder {
    pub(crate) proto: PolygonEdge2,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
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
