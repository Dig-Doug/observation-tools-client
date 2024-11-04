use crate::artifacts::Geometry2Builder;
use crate::artifacts::Object2Builder;
use crate::artifacts::Point2Builder;
use crate::artifacts::PolygonEdge2Builder;
use itertools::Itertools;
use observation_tools_common::proto::Polygon2;
use wasm_bindgen::prelude::*;

/// A 2D polygon. Polygon2s are represented as an edge-loop, so an edge will be
/// automatically created between the last and first vertex.
#[doc = docify::embed_run!("tests/examples.rs", polygon2_example)]
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Polygon2Builder {
    pub(crate) proto: Polygon2,
}

#[wasm_bindgen]
impl Polygon2Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon2Builder {
        Polygon2Builder {
            proto: Polygon2::default(),
        }
    }

    /// Add a vertex to the polygon.
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
        Polygon2Builder {
            proto: Polygon2 {
                edges: edges.iter().map(|edge| edge.proto.clone()).collect(),
            },
        }
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
