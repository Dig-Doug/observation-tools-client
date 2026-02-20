//! Blob storage for observation payloads

use super::StorageError;
use super::StorageResult;
use bytes::Bytes;
use object_store::local::LocalFileSystem;
use object_store::path::Path as ObjectPath;
use object_store::ObjectStore;
use observation_tools_shared::ObservationId;
use observation_tools_shared::PayloadId;
use std::path::Path;
use std::sync::Arc;

/// Trait for storing and retrieving blob data
#[async_trait::async_trait]
pub trait BlobStorage: Send + Sync {
  /// Store blob data for an observation payload
  async fn store_blob(
    &self,
    obs_id: ObservationId,
    payload_id: PayloadId,
    data: Bytes,
  ) -> StorageResult<()>;

  /// Retrieve blob data for an observation payload
  async fn get_blob(&self, obs_id: ObservationId, payload_id: PayloadId) -> StorageResult<Bytes>;
}

/// Object store-based blob storage
pub struct LocalBlobStorage {
  store: Arc<dyn ObjectStore>,
}

impl LocalBlobStorage {
  /// Create a new local blob storage instance
  pub fn new(path: impl AsRef<Path>) -> StorageResult<Self> {
    let path = path.as_ref();

    // Ensure the directory exists
    std::fs::create_dir_all(path)?;

    let store =
      LocalFileSystem::new_with_prefix(path).map_err(|e| StorageError::Internal(e.to_string()))?;

    Ok(Self {
      store: Arc::new(store),
    })
  }

  /// Convert observation ID + payload ID to object path
  fn id_to_path(&self, obs_id: ObservationId, payload_id: PayloadId) -> ObjectPath {
    ObjectPath::from(format!("{}/{}", obs_id, payload_id.as_str()))
  }
}

#[async_trait::async_trait]
impl BlobStorage for LocalBlobStorage {
  async fn store_blob(
    &self,
    obs_id: ObservationId,
    payload_id: PayloadId,
    data: Bytes,
  ) -> StorageResult<()> {
    let path = self.id_to_path(obs_id, payload_id);

    self
      .store
      .put(&path, data.into())
      .await
      .map_err(|e| StorageError::Internal(e.to_string()))?;

    Ok(())
  }

  async fn get_blob(&self, obs_id: ObservationId, payload_id: PayloadId) -> StorageResult<Bytes> {
    let path = self.id_to_path(obs_id, payload_id);

    let result = self
      .store
      .get(&path)
      .await
      .map_err(|e| StorageError::NotFound(format!("Blob not found: {}", e)))?;

    let bytes = result
      .bytes()
      .await
      .map_err(|e| StorageError::Internal(e.to_string()))?;

    Ok(bytes)
  }
}
