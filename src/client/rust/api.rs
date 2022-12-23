use artifacts_api_rust_proto::{StructuredData, Image2};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use crate::artifact_uploader_2d::Type2d;

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
        proto.set_data(data.to_vec());
        Image2Builder { proto }
    }
}

impl Into<StructuredData> for &Image2Builder {
    fn into(self) -> StructuredData {
        let mut s = StructuredData::new();
        s.set_image2(self.proto.clone());
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