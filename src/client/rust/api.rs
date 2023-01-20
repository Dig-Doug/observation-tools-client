use artifacts_api_rust_proto::{StructuredData, Image2, Sphere, Geometry3, Number, ArtifactId, Object3};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use crate::util::{encode_id_proto, new_uuid_proto};

pub struct NumberBuilder {
    pub(crate) proto: Number,
}

impl Into<NumberBuilder> for f64 {
    fn into(self) -> NumberBuilder {
        let mut proto = Number::new();
        proto.d = self;
        NumberBuilder { proto }
    }
}

#[cfg_attr(feature = "python", pyclass)]
pub struct Image2Builder {
    pub(crate) proto: Image2,
}

#[cfg_attr(feature = "python", pymethods)]
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

pub struct SphereBuilder {
    pub(crate) proto: Sphere,
}

impl SphereBuilder {
    pub fn new(radius: impl Into<NumberBuilder>) -> SphereBuilder {
        let mut proto = Sphere::new();
        proto.radius = Some(radius.into().proto).into();
        SphereBuilder { proto }
    }
}

impl Into<Geometry3Builder> for &SphereBuilder {
    fn into(self) -> Geometry3Builder {
        Geometry3Builder::sphere(self)
    }
}

impl Into<StructuredData> for &SphereBuilder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        *s.mut_sphere() = self.proto.clone();
        s
    }
}

pub struct Geometry3Builder {
    pub(crate) proto: Geometry3,
}

impl Geometry3Builder {
    pub fn sphere(sphere: &SphereBuilder) -> Geometry3Builder {
        let mut proto = Geometry3::new();
        *proto.mut_sphere() = sphere.proto.clone();
        Geometry3Builder { proto }
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