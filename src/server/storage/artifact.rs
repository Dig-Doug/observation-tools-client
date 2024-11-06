use crate::storage::sqlite::SqliteArtifactStorage;
use crate::storage::ArtifactVersion;
use futures_util::TryStream;
use std::error::Error;
use tokio_util::bytes::Bytes;

// Note: It would be nice to use a trait and have separate implementations, but
// it's not easy with the `impl TryStream<>`
#[derive(Clone)]
pub enum ArtifactStorage {
    Local(SqliteArtifactStorage),
}

impl ArtifactStorage {
    pub async fn write_artifact_version<E: Error + Send + Sync + 'static>(
        &self,
        version: ArtifactVersion,
        bytes_stream: impl TryStream<Ok = Bytes, Error = E> + Unpin,
    ) -> Result<(), anyhow::Error> {
        match self {
            Self::Local(storage) => storage.write_artifact_version(version, bytes_stream).await,
        }
    }
}
