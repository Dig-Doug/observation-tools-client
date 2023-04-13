use crate::util::encode_id_proto;
use crate::util::new_uuid_proto;
use artifacts_api_rust_proto::ArtifactId;
#[cfg(feature = "python")]
use pyo3::prelude::*;

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
