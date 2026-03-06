//! Group builder and handle types for hierarchical observation grouping

use crate::client::UploaderMessage;
use crate::context;
use crate::execution::ExecutionHandle;
use crate::observation::ObservationBuilder;
use crate::observation_handle::SendObservation;
use crate::Error;
use observation_tools_shared::GroupId;
use observation_tools_shared::LogLevel;
use observation_tools_shared::ObservationId;
use observation_tools_shared::ObservationType;
use observation_tools_shared::Payload;
use observation_tools_shared::SourceInfo;
use std::collections::HashMap;

/// Builder for creating groups
///
/// Groups are first-class hierarchical containers for observations.
/// They are themselves observations with `ObservationType::Group`.
pub struct GroupBuilder {
  name: String,
  custom_id: Option<GroupId>,
  parent_group_id: Option<GroupId>,
  metadata: HashMap<String, String>,
  source: Option<SourceInfo>,
  log_level: Option<LogLevel>,
}

impl GroupBuilder {
  /// Create a new group builder with the given name
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      custom_id: None,
      parent_group_id: None,
      metadata: HashMap::new(),
      source: None,
      log_level: None,
    }
  }

  /// Set a custom group ID
  pub fn id(mut self, id: impl Into<String>) -> Self {
    self.custom_id = Some(GroupId::from(id.into()));
    self
  }

  /// Set the parent group ID (for creating child groups)
  pub(crate) fn parent(mut self, parent_id: GroupId) -> Self {
    self.parent_group_id = Some(parent_id);
    self
  }

  /// Add metadata to the group
  pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.metadata.insert(key.into(), value.into());
    self
  }

  /// Set the source location for the group
  pub fn source(mut self, file: impl Into<String>, line: u32) -> Self {
    self.source = Some(SourceInfo {
      file: file.into(),
      line,
      column: None,
    });
    self
  }

  /// Set the log level for the group
  pub(crate) fn log_level(mut self, level: LogLevel) -> Self {
    self.log_level = Some(level);
    self
  }

  /// Create a GroupBuilder pre-configured for a tracing span
  pub fn from_span(
    name: impl Into<String>,
    log_level: LogLevel,
    parent_group_id: Option<GroupId>,
  ) -> Self {
    let mut builder = Self::new(name).log_level(log_level);

    if let Some(parent_id) = parent_group_id {
      builder = builder.parent(parent_id);
    }

    builder
  }

  /// Build and send the group using the current execution context
  pub fn build(self) -> SendGroup {
    match context::get_current_execution() {
      Some(execution) => self.build_with_execution(&execution),
      None => {
        log::trace!(
          "No execution context available for group '{}'",
          self.name
        );
        SendGroup::stub(Error::NoExecutionContext)
      }
    }
  }

  /// Build and send the group with an explicit execution handle
  pub fn build_with_execution(self, execution: &ExecutionHandle) -> SendGroup {
    let group_id = self.custom_id.unwrap_or_else(GroupId::new);
    let observation_id = ObservationId::new();

    let group_handle = GroupHandle {
      group_id: group_id.clone(),
      execution_id: execution.id(),
      uploader_tx: execution.uploader_tx.clone(),
      base_url: execution.base_url().to_string(),
    };

    // Serialize metadata as the payload
    let payload = if self.metadata.is_empty() {
      Payload::json("{}".to_string())
    } else {
      Payload::json(serde_json::to_string(&self.metadata).unwrap_or_else(|_| "{}".to_string()))
    };

    let mut builder = ObservationBuilder::new(self.name)
      .with_id(observation_id)
      .observation_type(ObservationType::Group)
      .log_level(self.log_level.unwrap_or(LogLevel::Info))
      .with_group_id(group_id.clone())
      .execution(execution);

    if let Some(source) = self.source {
      builder = builder.source(source.file, source.line);
    }

    if let Some(parent_id) = self.parent_group_id {
      builder = builder.parent_group(parent_id);
    }
    for (k, v) in self.metadata {
      builder = builder.metadata(k, v);
    }

    let send = builder.send_observation(payload);
    SendGroup::from_send_observation(group_handle, send)
  }
}

/// Handle to a created group
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GroupHandle {
  pub(crate) group_id: GroupId,
  pub(crate) execution_id: observation_tools_shared::models::ExecutionId,
  pub(crate) uploader_tx: async_channel::Sender<UploaderMessage>,
  pub(crate) base_url: String,
}

impl GroupHandle {
  /// Get the group ID
  pub fn id(&self) -> GroupId {
    self.group_id.clone()
  }

  /// Create a child group builder with this group as parent
  pub fn child(&self, name: impl Into<String>) -> GroupBuilder {
    GroupBuilder::new(name).parent(self.group_id.clone())
  }

  /// Construct a GroupHandle from a known ID without creating/sending a group.
  ///
  /// This is useful for the tracing layer which already knows span IDs
  /// and doesn't need to create group observations for every span.
  pub fn from_id(group_id: GroupId, execution: &ExecutionHandle) -> Self {
    Self {
      group_id,
      execution_id: execution.id(),
      uploader_tx: execution.uploader_tx.clone(),
      base_url: execution.base_url().to_string(),
    }
  }
}

/// Result of sending a group, allowing waiting for upload
pub struct SendGroup {
  group_handle: GroupHandle,
  send: SendObservation,
}

impl SendGroup {
  fn stub(error: Error) -> Self {
    Self {
      group_handle: GroupHandle {
        group_id: GroupId::new(),
        execution_id: observation_tools_shared::models::ExecutionId::nil(),
        uploader_tx: async_channel::unbounded().0,
        base_url: String::new(),
      },
      send: SendObservation::stub(error),
    }
  }

  pub(crate) fn from_send_observation(group_handle: GroupHandle, send: SendObservation) -> Self {
    Self { group_handle, send }
  }

  /// Wait for the group to be uploaded
  pub async fn wait_for_upload(mut self) -> crate::error::Result<GroupHandle> {
    self.send.wait_for_upload().await?;
    Ok(self.group_handle)
  }

  /// Get a reference to the group handle
  pub fn handle(&self) -> &GroupHandle {
    &self.group_handle
  }

  /// Consume and return the group handle
  pub fn into_handle(self) -> GroupHandle {
    self.group_handle
  }
}
