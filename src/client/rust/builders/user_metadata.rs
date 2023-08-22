use crate::generated::ArtifactUserMetadata;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UserMetadataBuilder {
    pub(crate) proto: ArtifactUserMetadata,
}

#[wasm_bindgen]
impl UserMetadataBuilder {
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
        UserMetadataBuilder::new(self.into())
    }
}

impl Into<UserMetadataBuilder> for String {
    fn into(self) -> UserMetadataBuilder {
        UserMetadataBuilder::new(&self)
    }
}
