use crate::artifacts::Geometry3Builder;
use crate::artifacts::Object3Builder;
use crate::artifacts::Point3Builder;
use crate::artifacts::PolygonEdge3Builder;
use crate::generated::Polygon3;
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

/// A 3D polygon within a single plane.
#[cfg_attr(feature = "wasm", derive(TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Polygon3Builder {
    pub(crate) proto: Polygon3,
}

#[wasm_bindgen]
impl Polygon3Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon3Builder {
        let proto = Polygon3::new();
        Polygon3Builder { proto }
    }

    pub fn add_edge(&mut self, edge: PolygonEdge3Builder) {
        self.proto.edges.push(edge.proto);
    }
}

impl Polygon3Builder {
    pub fn from_points<T: Into<Point3Builder>>(points: Vec<T>) -> Polygon3Builder {
        Polygon3Builder::from_edges(
            &points
                .into_iter()
                .map(|point| PolygonEdge3Builder::new(point.into()))
                .collect_vec(),
        )
    }

    pub fn from_edges(edges: &[PolygonEdge3Builder]) -> Polygon3Builder {
        let mut proto = Polygon3::new();
        proto.edges = edges.iter().map(|edge| edge.proto.clone()).collect();
        Polygon3Builder { proto }
    }
}

impl Into<Geometry3Builder> for Polygon3Builder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::polygon(self)
    }
}

impl Into<Object3Builder> for Polygon3Builder {
    fn into(self) -> Object3Builder {
        let builder = Object3Builder::new(self.into());
        builder
    }
}
