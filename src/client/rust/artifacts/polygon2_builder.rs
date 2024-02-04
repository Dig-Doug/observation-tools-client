use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Point2Builder;
use crate::artifacts::PolygonEdge2Builder;

use crate::generated::Polygon2;
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 2D polygon.
#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
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
    pub fn from_points<T: Into<Point2Builder>>(points: Vec<T>) -> Polygon2Builder {
        Polygon2Builder::from_edges(
            &points
                .into_iter()
                .map(|point| PolygonEdge2Builder::new(point.into()))
                .collect_vec(),
        )
    }

    pub fn from_edges(edges: &[PolygonEdge2Builder]) -> Polygon2Builder {
        let mut proto = Polygon2::new();
        proto.edges = edges.iter().map(|edge| edge.proto.clone()).collect();
        Polygon2Builder { proto }
    }
}

impl Into<Geometry2Builder> for Polygon2Builder {
    fn into(self) -> Geometry2Builder {
        Geometry2Builder::polygon(self)
    }
}

impl Into<Object2Builder> for Polygon2Builder {
    fn into(self) -> Object2Builder {
        let builder = Object2Builder::new(self.into());
        builder
    }
}
