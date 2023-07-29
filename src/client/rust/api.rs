use crate::util::new_uuid_proto;
use artifacts_api_rust_proto::ArtifactId;

pub(crate) fn new_artifact_id() -> ArtifactId {
    let mut id = ArtifactId::new();
    id.uuid = Some(new_uuid_proto()).into();
    id
}
