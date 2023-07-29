use artifacts_api_rust_proto::CreateArtifactRequest;
#[cfg(feature = "files")]
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct UploadArtifactTask {
    pub request: CreateArtifactRequest,
    pub payload: Option<UploadArtifactTaskPayload>,
}

#[derive(Debug)]
pub enum UploadArtifactTaskPayload {
    #[cfg(feature = "files")]
    File(NamedTempFile),
    Bytes(Vec<u8>),
}
