use artifacts_api_rust_proto::ArtifactUserMetadata;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct UserMetadataBuilder {
    pub(crate) proto: ArtifactUserMetadata,
}

#[wasm_bindgen]
impl UserMetadataBuilder {
    #[cfg(not(feature = "python"))]
    #[wasm_bindgen(constructor)]
    pub fn new_js(name: String) -> Self {
        Self::new_impl(&name)
    }

    pub fn new(name: &str) -> Self {
        Self::new_impl(name)
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.proto.metadata.insert(key, value);
    }
}

impl UserMetadataBuilder {
    fn new_impl(name: &str) -> UserMetadataBuilder {
        let mut proto = ArtifactUserMetadata::new();
        proto.name = name.to_string();
        UserMetadataBuilder { proto }
    }
}

impl Into<UserMetadataBuilder> for &str {
    fn into(self) -> UserMetadataBuilder {
        UserMetadataBuilder::new(self)
    }
}
