use crate::artifacts::Geometry3Builder;
use crate::artifacts::Object3Builder;
use crate::artifacts::Point3Builder;
use crate::artifacts::PolygonEdge3Builder;
use itertools::Itertools;
use observation_tools_common::proto::Polygon3;
use wasm_bindgen::prelude::*;

/// A 3D polygon within a single plane.
#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Polygon3Builder {
    pub(crate) proto: Polygon3,
}

#[wasm_bindgen]
impl Polygon3Builder {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon3Builder {
        Polygon3Builder {
            proto: Polygon3::default(),
        }
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
        Polygon3Builder {
            proto: Polygon3 {
                edges: edges.iter().map(|edge| edge.proto.clone()).collect(),
            },
        }
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
