use crate::auth::principal::Principal;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use observation_tools_common::project::ProjectId;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tracing::warn;

pub trait ResourceId: Debug + Clone + Hash + Sync + Send + Eq {}

impl ResourceId for ProjectId {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Permission<T: ResourceId> {
    pub principal: Principal,
    pub resource_id: T,
    pub operation: Operation,
}

impl<T: ResourceId> Permission<T> {
    pub fn new(principal: Principal, resource_id: T, operation: Operation) -> Self {
        Permission {
            principal,
            resource_id,
            operation,
        }
    }

    pub fn from_ids(principal: Principal, resource_ids: Vec<T>, operation: Operation) -> Vec<Self> {
        resource_ids
            .into_iter()
            .map(|key| Permission::new(principal.clone(), key, operation))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operation {
    Read,
    Write,
    Owner,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessResult {
    Allow,
    Deny,
}

pub type PermissionDataLoader = Arc<DataLoader<PermissionLoader, HashMapCache>>;

pub struct PermissionLoader {}

impl<T: ResourceId + 'static> Loader<Permission<T>> for PermissionLoader {
    type Value = AccessResult;
    type Error = String;

    async fn load(
        &self,
        keys: &[Permission<T>],
    ) -> Result<HashMap<Permission<T>, Self::Value>, Self::Error> {
        warn!("TODO(doug): PermissionLoader not implemented");
        Ok(keys
            .iter()
            .map(|key| (key.clone(), AccessResult::Allow))
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct PermissionStorage {}

impl PermissionStorage {
    pub async fn create_permission<T>(&self, permission: Permission<T>) -> Result<(), String>
    where
        T: ResourceId,
    {
        warn!("TODO(doug): PermissionStorage not implemented");
        Ok(())
    }
}
