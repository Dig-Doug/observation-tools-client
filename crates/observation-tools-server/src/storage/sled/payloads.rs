//! Payload storage methods for SledStorage

use super::keys::PayloadKey;
use super::SledStorage;
use crate::storage::proto::StoredPayloadEntry;
use crate::storage::ObservationPayloadPage;
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
    execution_id: ExecutionId,
    obs_id: &ObservationId,
    payloads: &[StoredPayload],
  ) -> StorageResult<()> {
    let payload_tree = self.payloads_tree()?;
    for payload in payloads {
      let is_blob = matches!(payload.data, PayloadData::Blob);
      let pkey = PayloadKey {
        execution_id: &execution_id,
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

  async fn get_all_payloads(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
  ) -> StorageResult<Vec<StoredPayload>> {
    let mut all = Vec::new();
    let mut page_token = None;
    loop {
      let page = self
        .get_payloads(execution_id, observation_id, page_token)
        .await?;
      all.extend(page.payloads);
      match page.pagination.next_page_token {
        Some(token) => page_token = Some(token),
        None => break,
      }
    }
    Ok(all)
  }

  async fn get_payload(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
    payload_id: PayloadId,
  ) -> StorageResult<StoredPayload> {
    let payload_tree = self.payloads_tree()?;
    let pkey = PayloadKey {
      execution_id: &execution_id,
      observation_id: &observation_id,
      payload_id: &payload_id,
    }
    .encode();
    let value = payload_tree
      .get(pkey.as_bytes())?
      .ok_or_else(|| {
        StorageError::NotFound(format!(
          "Payload {} not found for observation {}",
          payload_id.as_str(),
          observation_id
        ))
      })?;
    let entry = StoredPayloadEntry::decode(value.as_ref())?;
    Ok(into_stored_payload(payload_id, entry))
  }

  async fn get_payloads(
    &self,
    execution_id: ExecutionId,
    observation_id: ObservationId,
    page_token: Option<String>,
  ) -> StorageResult<ObservationPayloadPage> {
    let payload_tree = self.payloads_tree()?;
    let prefix = PayloadKey::encode_prefix(&execution_id, &observation_id);

    let start_key = match page_token {
      Some(ref token) => format!("{}{}\x00", prefix, token),
      None => prefix.clone(),
    };
    let mut payloads = Vec::new();
    for item in payload_tree
      .range(start_key.as_bytes()..)
      .take_while({
        let prefix = prefix.clone();
        move |item| {
          item
            .as_ref()
            .map(|(k, _)| k.starts_with(prefix.as_bytes()))
            .unwrap_or(false)
        }
      })
      .take(PAGE_SIZE + 1)
    {
      let (key_bytes, value) = item?;
      let key_str = String::from_utf8(key_bytes.to_vec())
        .map_err(|e| StorageError::Internal(format!("Invalid key encoding: {}", e)))?;
      let payload_id_str = PayloadKey::parse_payload_id(&key_str, &prefix)
        .ok_or_else(|| StorageError::Internal("Payload key missing prefix".to_string()))?;

      let entry = StoredPayloadEntry::decode(value.as_ref())?;
      payloads.push(into_stored_payload(PayloadId::from(payload_id_str), entry));
    }

    let next_page_token = if payloads.len() > PAGE_SIZE {
      payloads.pop();
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

fn into_stored_payload(id: PayloadId, entry: StoredPayloadEntry) -> StoredPayload {
  let data = if entry.is_blob {
    PayloadData::Blob
  } else {
    PayloadData::Inline(entry.data)
  };
  StoredPayload {
    id,
    name: entry.name,
    mime_type: entry.mime_type,
    size: entry.size as usize,
    data,
  }
}

#[cfg(test)]
mod tests {
  use super::super::test_helpers::*;
  use crate::storage::{ExecutionStorage, ObservationStorage, PayloadStorage};
  use crate::storage::PayloadData;
  use crate::storage::StoredPayload;
  use observation_tools_shared::ExecutionId;
  use observation_tools_shared::PayloadId;

  fn make_test_payload(name: &str, data: &[u8]) -> StoredPayload {
    StoredPayload {
      id: PayloadId::new(),
      name: name.to_string(),
      mime_type: "text/plain".to_string(),
      size: data.len(),
      data: PayloadData::Inline(data.to_vec()),
    }
  }

  async fn setup_with_payloads(
    count: usize,
  ) -> anyhow::Result<(
    super::SledStorage,
    tempfile::TempDir,
    ExecutionId,
    observation_tools_shared::ObservationId,
    Vec<StoredPayload>,
  )> {
    let (storage, dir) = test_storage();
    let exec = make_execution();
    let exec_id = exec.id;
    storage.store_execution(&exec).await?;
    let obs = make_observation(exec_id, "with-payloads");
    let obs_id = obs.id;
    storage.store_observations(vec![obs]).await?;

    let payloads: Vec<_> = (0..count)
      .map(|i| make_test_payload(&format!("payload-{}", i), format!("data-{}", i).as_bytes()))
      .collect();
    storage.store_payloads(exec_id, &obs_id, &payloads).await?;
    Ok((storage, dir, exec_id, obs_id, payloads))
  }

  #[tokio::test]
  async fn test_get_payloads() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, _) = setup_with_payloads(2).await?;

    let page = storage
      .get_payloads(exec_id, obs_id, None)
      .await?;
    assert_eq!(page.payloads.len(), 2);
    assert!(matches!(page.payloads[0].data, PayloadData::Inline(_)));
    assert!(matches!(page.payloads[1].data, PayloadData::Inline(_)));
    assert!(page.pagination.next_page_token.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_payload_by_id() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, payloads) = setup_with_payloads(2).await?;

    let result = storage
      .get_payload(exec_id, obs_id, payloads[0].id.clone())
      .await?;
    assert_eq!(result.name, "payload-0");
    assert!(matches!(result.data, PayloadData::Inline(ref d) if d == b"data-0"));
    Ok(())
  }

  #[tokio::test]
  async fn test_get_payload_not_found() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, _) = setup_with_payloads(0).await?;

    let result = storage.get_payload(exec_id, obs_id, PayloadId::new()).await;
    assert!(result.is_err());
    Ok(())
  }

  #[tokio::test]
  async fn test_get_all_payloads() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, payloads) = setup_with_payloads(3).await?;

    let all = storage.get_all_payloads(exec_id, obs_id).await?;
    assert_eq!(all.len(), 3);
    let names: Vec<_> = all.iter().map(|p| p.name.as_str()).collect();
    for p in &payloads {
      assert!(names.contains(&p.name.as_str()));
    }
    Ok(())
  }

  #[tokio::test]
  async fn test_get_payloads_pagination() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, _) = setup_with_payloads(crate::storage::PAGE_SIZE + 5).await?;

    let page1 = storage
      .get_payloads(exec_id, obs_id, None)
      .await?;
    assert_eq!(page1.payloads.len(), crate::storage::PAGE_SIZE);
    assert!(page1.pagination.next_page_token.is_some());
    assert!(page1.pagination.previous_page_token.is_none());

    let page2 = storage
      .get_payloads(exec_id, obs_id, page1.pagination.next_page_token)
      .await?;
    assert_eq!(page2.payloads.len(), 5);
    assert!(page2.pagination.next_page_token.is_none());
    assert!(page2.pagination.previous_page_token.is_some());

    // No overlap between pages
    let page1_ids: std::collections::HashSet<_> =
      page1.payloads.iter().map(|p| p.id.clone()).collect();
    for p in &page2.payloads {
      assert!(!page1_ids.contains(&p.id));
    }
    Ok(())
  }

  #[tokio::test]
  async fn test_get_all_payloads_spans_pages() -> anyhow::Result<()> {
    let count = crate::storage::PAGE_SIZE + 5;
    let (storage, _dir, exec_id, obs_id, _) = setup_with_payloads(count).await?;

    let all = storage.get_all_payloads(exec_id, obs_id).await?;
    assert_eq!(all.len(), count);
    Ok(())
  }

  #[tokio::test]
  async fn test_store_and_get_blob_payload() -> anyhow::Result<()> {
    let (storage, _dir, exec_id, obs_id, _) = setup_with_payloads(0).await?;

    let blob_payload = StoredPayload {
      id: PayloadId::new(),
      name: "blob".to_string(),
      mime_type: "application/octet-stream".to_string(),
      size: 1024,
      data: PayloadData::Blob,
    };
    storage
      .store_payloads(exec_id, &obs_id, &[blob_payload.clone()])
      .await?;

    let result = storage
      .get_payload(exec_id, obs_id, blob_payload.id.clone())
      .await?;
    assert_eq!(result.name, "blob");
    assert!(matches!(result.data, PayloadData::Blob));
    assert_eq!(result.size, 1024);
    Ok(())
  }
}
