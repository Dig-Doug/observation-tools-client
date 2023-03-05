use crate::sphere_builder::SphereBuilder;
use crate::util::{encode_id_proto, new_uuid_proto};
use artifacts_api_rust_proto::{
    ArtifactId, Geometry3, Image2, Number, Object3, Sphere, StructuredData,
};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl Image2Builder {
    #[cfg(not(feature = "python"))]
    pub fn new(data: &[u8]) -> Self {
        Self::new_impl(data)
    }

    // TODO(doug): Figure out why this doesn't work: #[cfg_attr(feature = "python", new)]
    #[cfg(feature = "python")]
    #[new]
    pub fn new(data: &[u8]) -> Self {
        Self::new_impl(data)
    }
}

impl Image2Builder {
    fn new_impl(data: &[u8]) -> Image2Builder {
        let mut proto = Image2::new();
        proto.data = data.to_vec();
        Image2Builder { proto }
    }
}

impl Into<StructuredData> for &Image2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_image2() = self.proto.clone();
        s
    }
}

/*
impl Type2d for Image2Builder {
    fn convert_2d_to_raw(&self) -> StructuredData {
        self.into()
    }
}
 */

pub(crate) fn new_artifact_id() -> ArtifactId {
    let mut id = ArtifactId::new();
    id.uuid = Some(new_uuid_proto()).into();
    id
}

pub(crate) fn new_encoded_artifact_id() -> String {
    encode_id_proto(&new_artifact_id())
}
