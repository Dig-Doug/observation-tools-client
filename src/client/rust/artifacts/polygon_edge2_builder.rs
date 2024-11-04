use crate::artifacts::Point2Builder;
use observation_tools_common::proto::PolygonEdge2;
use wasm_bindgen::prelude::*;

/// An edge of a [Polygon2Builder](crate::artifacts::Polygon2Builder).
#[wasm_bindgen]
pub struct PolygonEdge2Builder {
    pub(crate) proto: PolygonEdge2,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl PolygonEdge2Builder {
    pub fn new_js(vertex: Point2Builder) -> PolygonEdge2Builder {
        PolygonEdge2Builder::new(vertex)
    }
}

impl PolygonEdge2Builder {
    pub fn new(vertex: impl Into<Point2Builder>) -> PolygonEdge2Builder {
        PolygonEdge2Builder {
            proto: PolygonEdge2 {
                vertex: Some(vertex.into().proto),
            },
        }
    }
}
