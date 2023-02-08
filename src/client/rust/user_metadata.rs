use artifacts_api_rust_proto::ArtifactUserMetadata;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[wasm_bindgen]
pub struct UserMetadataBuilder {
    pub(crate) proto: ArtifactUserMetadata,
}

pub fn new_user_metadata(name: String) -> Box<UserMetadataBuilder> {
    Box::new(UserMetadataBuilder::new(name))
}

fn user_metadata_from_name(name: &str) -> ArtifactUserMetadata {
    let mut metadata = ArtifactUserMetadata::new();
    metadata.name = name.to_string();
    metadata
}

#[cfg_attr(feature = "python", pymethods)]
#[wasm_bindgen]
impl UserMetadataBuilder {
    #[cfg(not(feature = "python"))]
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Self::new_impl(name)
    }

    // TODO(doug): Figure out why this doesn't work: #[cfg_attr(feature = "python", new)]
    #[cfg(feature = "python")]
    #[new]
    pub fn new(name: String) -> Self {
        Self::new_impl(name)
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.proto.metadata.insert(key, value);
    }
}

impl UserMetadataBuilder {
    fn new_impl(name: String) -> UserMetadataBuilder {
        let mut proto = ArtifactUserMetadata::new();
        proto.name = name;
        UserMetadataBuilder { proto }
    }
}
