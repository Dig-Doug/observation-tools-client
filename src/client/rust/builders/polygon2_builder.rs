use crate::builders::Geometry2Builder;
use crate::builders::PolygonEdge2Builder;
use artifacts_api_rust_proto::Polygon2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Polygon2Builder {
    pub(crate) proto: Polygon2,
}

#[wasm_bindgen]
impl Polygon2Builder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Polygon2Builder {
        let proto = Polygon2::new();
        Polygon2Builder { proto }
    }

    pub fn add_edge(&mut self, edge: PolygonEdge2Builder) {
        self.proto.edges.push(edge.proto);
    }
}

impl Polygon2Builder {
    pub fn from_edges(edges: &[PolygonEdge2Builder]) -> Polygon2Builder {
        let mut proto = Polygon2::new();
        proto.edges = edges.iter().map(|edge| edge.proto.clone()).collect();
        Polygon2Builder { proto }
    }
}

impl Into<Geometry2Builder> for &Polygon2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::polygon(self)
    }
}
