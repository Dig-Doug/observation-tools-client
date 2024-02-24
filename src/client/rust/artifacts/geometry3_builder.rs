use crate::artifacts::MeshBuilder;
use crate::artifacts::Polygon3Builder;
use crate::artifacts::SphereBuilder;
use crate::generated::Geometry3;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", derive(wasm_bindgen_derive::TryFromJsValue))]
#[wasm_bindgen]
#[derive(Clone)]
pub struct Geometry3Builder {
    pub(crate) proto: Geometry3,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "Geometry3Builder | SphereBuilder | Polygon3Builder | MeshBuilder"
    )]
    pub type IntoGeometry3Builder;
}

#[wasm_bindgen]
impl Geometry3Builder {
    pub(crate) fn sphere(sphere: SphereBuilder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_sphere() = sphere.proto;
        Geometry3Builder { proto }
    }

    pub(crate) fn polygon(polygon: Polygon3Builder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_polygon() = polygon.proto;
        Geometry3Builder { proto }
    }

    pub(crate) fn mesh(mesh: MeshBuilder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_mesh() = mesh.proto;
        Geometry3Builder { proto }
    }
}
