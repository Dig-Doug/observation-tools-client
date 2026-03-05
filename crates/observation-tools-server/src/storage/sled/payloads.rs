//! Payload storage methods for SledStorage

use super::keys::PayloadKey;
use super::SledStorage;
use crate::storage::proto::StoredObservation;
use crate::storage::proto::StoredPayloadEntry;
use crate::storage::ObservationPayloadPage;
use crate::storage::ObservationWithPayloads;
use crate::storage::PaginationInfo;
use crate::storage::PayloadData;
use crate::storage::PayloadStorage;
use crate::storage::StorageError;
use crate::storage::StorageResult;
use crate::storage::StoredPayload;
use crate::storage::PAGE_SIZE;
use observation_tools_shared::ExecutionId;
use observation_tools_shared::ObservationId;
use observation_tools_shared::PayloadId;
use prost::Message;

#[async_trait::async_trait]
impl PayloadStorage for SledStorage {
  async fn store_payloads(
    &self,
    obs_id: &ObservationId,
    payloads: &[StoredPayload],
  ) -> StorageResult<()> {
    let payload_tree = self.payloads_tree()?;
    self.insert_payloads(&payload_tree, obs_id, payloads)
  }

  async fn get_all_payloads(
    &self,
    observation_id: ObservationId,
  ) -> StorageResult<Vec<StoredPayload>> {
    self.scan_payloads(&observation_id, true)
  }

  async fn get_payloads(
    &self,
    _execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage> {
    let all_payloads = self.scan_payloads(&observation_id, true)?;

    // Filter payloads using cursor (payload_id)
    let mut payloads: Vec<StoredPayload> = if let Some(ref token) = page_token {
      all_payloads
        .into_iter()
        .skip_while(|p| p.id.as_str() <= token.as_str())
        .collect()
    } else {
      all_payloads
    };

    let next_page_token = if payloads.len() > PAGE_SIZE {
      payloads.truncate(PAGE_SIZE);
      payloads.last().map(|p| p.id.as_str().to_string())
    } else {
      None
    };

    let item_count = payloads.len();
    Ok(ObservationPayloadPage {
      payloads,
      pagination: PaginationInfo {
        item_count,
        previous_page_token: page_token,
        next_page_token,
      },
    })
  }
}

impl SledStorage {
  /// Scan the payloads tree for entries belonging to an observation.
  /// If `include_inline_data` is false, all payloads are returned with `PayloadData::Blob`.
  fn scan_payloads(
    &self,
    obs_id: &ObservationId,
    include_inline_data: bool,
  ) -> StorageResult<Vec<StoredPayload>> {
    let payload_tree = self.payloads_tree()?;
    let prefix = PayloadKey::encode_prefix(obs_id);
    let mut payloads = Vec::new();

    for item in payload_tree.scan_prefix(prefix.as_bytes()) {
      let (key_bytes, value) = item?;
      let key_str = String::from_utf8(key_bytes.to_vec())
        .map_err(|e| StorageError::Internal(format!("Invalid key encoding: {}", e)))?;
      let payload_id_str = PayloadKey::parse_payload_id(&key_str, &prefix)
        .ok_or_else(|| StorageError::Internal("Payload key missing prefix".to_string()))?;

      let entry = StoredPayloadEntry::decode(value.as_ref())?;
      let data = if !include_inline_data || entry.is_blob {
        PayloadData::Blob
      } else {
        PayloadData::Inline(entry.data)
      };

      payloads.push(StoredPayload {
        id: PayloadId::from(payload_id_str),
        name: entry.name,
        mime_type: entry.mime_type,
        size: entry.size as usize,
        data,
      });
    }

    Ok(payloads)
  }

  /// Decode a stored observation from raw bytes and scan the payloads tree for
  /// payload metadata. All payloads are returned with `PayloadData::Blob`.
  pub(crate) fn decode_metadata_only(
    &self,
    obs_id: &ObservationId,
    value: &[u8],
  ) -> StorageResult<ObservationWithPayloads> {
    let stored = StoredObservation::decode(value)?;
    let observation = stored.to_observation()?;
    let payloads = self.scan_payloads(obs_id, false)?;

    Ok(ObservationWithPayloads {
      observation,
      payloads,
    })
  }

  /// Store all payloads for an observation into a specific payloads tree.
  fn insert_payloads(
    &self,
    payload_tree: &sled::Tree,
    obs_id: &ObservationId,
    payloads: &[StoredPayload],
  ) -> StorageResult<()> {
    for payload in payloads {
      let is_blob = matches!(payload.data, PayloadData::Blob);
      let pkey = PayloadKey {
        observation_id: obs_id,
        payload_id: &payload.id,
      }
      .encode();
      let entry = StoredPayloadEntry {
        name: payload.name.clone(),
        mime_type: payload.mime_type.clone(),
        size: payload.size as u64,
        is_blob,
        data: match &payload.data {
          PayloadData::Inline(data) => data.clone(),
          PayloadData::Blob => Vec::new(),
        },
      };
      payload_tree.insert(pkey.as_bytes(), entry.encode_to_vec())?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::super::test_helpers::*;
  use crate::storage::{ExecutionStorage, ObservationStorage, PayloadStorage};
  use crate::storage::PayloadData;
  use crate::storage::StoredPayload;
  use observation_tools_shared::PayloadId;

  #[tokio::test]
  async fn test_get_payloads() {
    let (storage, _dir) = test_storage();
    let exec = make_execution();
    storage.store_execution(&exec).await.expect("store exec");

    let obs = make_observation(exec.id, "with-payloads");
    let obs_id = obs.id;

    let payloads = vec![
      StoredPayload {
        id: PayloadId::new(),
        name: "payload-1".to_string(),
        mime_type: "text/plain".to_string(),
        size: 5,
        data: PayloadData::Inline(b"hello".to_vec()),
      },
      StoredPayload {
        id: PayloadId::new(),
        name: "payload-2".to_string(),
        mime_type: "text/plain".to_string(),
        size: 5,
        data: PayloadData::Inline(b"world".to_vec()),
      },
    ];

    storage
      .store_observations(vec![obs])
      .await
      .expect("store obs");
    storage
      .store_payloads(&obs_id, &payloads)
      .await
      .expect("store payloads");

    let page = storage
      .get_payloads(exec.id, obs_id, None)
      .await
      .expect("get payloads");
    assert_eq!(page.payloads.len(), 2);
    assert!(matches!(page.payloads[0].data, PayloadData::Inline(_)));
    assert!(matches!(page.payloads[1].data, PayloadData::Inline(_)));
  }
}
