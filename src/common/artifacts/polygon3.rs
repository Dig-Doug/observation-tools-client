use crate::artifacts::Geometry3;
use crate::artifacts::Object3;
use crate::artifacts::Point3;
use crate::artifacts::PolygonEdge3;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A 3D polygon within a single plane.
#[wasm_bindgen]
#[pyo3::pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Polygon3 {
    #[wasm_bindgen(skip)]
    pub edges: Vec<PolygonEdge3>,
}

#[wasm_bindgen]
impl Polygon3 {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Polygon3 {
        Polygon3 { edges: vec![] }
    }

    pub fn add_edge(&mut self, edge: PolygonEdge3) {
        self.edges.push(edge);
    }
}

// WASM only functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Polygon3 {
    pub fn into_object(self) -> Object3 {
        self.into()
    }
}

impl Polygon3 {
    pub fn from_points<T: Into<Point3>>(points: Vec<T>) -> Polygon3 {
        Polygon3 {
            edges: points
                .into_iter()
                .map(|point| PolygonEdge3::new(point.into()))
                .collect(),
        }
    }

    pub fn from_edges(edges: &[PolygonEdge3]) -> Polygon3 {
        Polygon3 {
            edges: edges.iter().map(|edge| edge.clone()).collect(),
        }
    }
}

impl Into<Geometry3> for Polygon3 {
    fn into(self) -> Geometry3 {
        Geometry3::Polygon3(self)
    }
}

impl Into<Object3> for Polygon3 {
    fn into(self) -> Object3 {
        let builder = Object3::new(self.into());
        builder
    }
}
