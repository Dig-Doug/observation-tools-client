use crate::artifacts::Geometry2;
use crate::artifacts::Object2;
use crate::artifacts::Point2;
use crate::artifacts::PolygonEdge2;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 2D polygon. Polygon2s are represented as an edge-loop, so an edge will be
/// automatically created between the last and first vertex.
//#[doc = docify::embed_run!("tests/examples.rs", polygon2_example)]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Polygon2 {
    #[wasm_bindgen(skip)]
    pub edges: Vec<PolygonEdge2>,
}

#[wasm_bindgen]
impl Polygon2 {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon2 {
        Polygon2 { edges: Vec::new() }
    }

    /// Add a vertex to the polygon.
    pub fn add_edge(&mut self, edge: PolygonEdge2) {
        self.edges.push(edge);
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Polygon2 {
    pub fn into_object(self) -> Object2 {
        self.into()
    }
}

impl Polygon2 {
    pub fn from_points<T: Into<Point2>>(points: Vec<T>) -> Polygon2 {
        Polygon2 {
            edges: points
                .into_iter()
                .map(|point| PolygonEdge2::new(point.into()))
                .collect(),
        }
    }

    pub fn from_edges(edges: &[PolygonEdge2]) -> Polygon2 {
        Polygon2 {
            edges: edges.iter().map(|edge| edge.clone()).collect(),
        }
    }
}

impl Into<Geometry2> for Polygon2 {
    fn into(self) -> Geometry2 {
        Geometry2::Polygon2(self)
    }
}

impl Into<Object2> for Polygon2 {
    fn into(self) -> Object2 {
        let builder = Object2::new(self.into());
        builder
    }
}
