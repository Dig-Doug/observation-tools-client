use crate::auth::principal::Principal;
use crate::auth::principal::PrincipalId;
use crate::auth::resource_id::ResourceId;
use crate::graphql::LoaderError;
use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::HashMapCache;
use async_graphql::dataloader::Loader;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission<T> {
    pub principal: PrincipalId,
    pub resource_id: T,
    pub operation: Operation,
}

pub trait IntoResourceId:
    Into<ResourceId> + Debug + Clone + Hash + Sync + Send + Eq + 'static
{
}

impl<T> Permission<T>
where
    T: IntoResourceId,
{
    pub fn new(principal: Principal, resource_id: T, operation: Operation) -> Self {
        Permission {
            principal: principal.id(),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessResult<T> {
    pub permission: Permission<T>,
    pub allow: bool,
}

pub type PermissionDataLoader = Arc<DataLoader<PermissionLoader, HashMapCache>>;

pub async fn load_permissions_and_filter_ids<T: IntoResourceId>(
    permission_data_loader: &PermissionDataLoader,
    principal: &Principal,
    keys: &[T],
) -> Result<(HashMap<T, AccessResult<T>>, Vec<T>), LoaderError> {
    let permissions = Permission::from_ids(principal.clone(), keys.to_vec(), Operation::Read);
    let mut access_results = permission_data_loader
        .load_many(permissions.clone())
        .await
        .map_err(|e| LoaderError::Error { message: e })?;

    let mut key_to_result: HashMap<T, AccessResult<T>> = HashMap::new();
    for (key, permission) in keys.iter().zip(permissions) {
        key_to_result.insert(
            key.clone(),
            access_results
                .remove(&permission)
                .unwrap_or_else(|| AccessResult {
                    permission,
                    allow: false,
                }),
        );
    }

    let ids_to_fetch: Vec<T> = key_to_result
        .iter()
        .filter(|(_, accessible)| accessible.allow)
        .map(|(key, _)| key.clone())
        .collect();
    Ok((key_to_result, ids_to_fetch))
}

pub struct PermissionLoader {}

impl<T> Loader<Permission<T>> for PermissionLoader
where
    T: IntoResourceId,
{
    type Value = AccessResult<T>;
    type Error = String;

    async fn load(
        &self,
        keys: &[Permission<T>],
    ) -> Result<HashMap<Permission<T>, Self::Value>, Self::Error> {
        warn!("TODO(doug): PermissionLoader not implemented");
        Ok(keys
            .iter()
            .map(|key| {
                (
                    key.clone(),
                    AccessResult {
                        permission: key.clone(),
                        allow: true,
                    },
                )
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct PermissionStorage {}

impl PermissionStorage {
    pub async fn create_permission<T>(&self, permission: Permission<T>) -> Result<(), String> {
        warn!("TODO(doug): PermissionStorage not implemented");
        Ok(())
    }

    pub async fn get_resources<T>(
        &self,
        principal: &Principal,
        operation: Operation,
        from: usize,
        count: usize,
    ) -> Result<Vec<T>, String> {
        warn!("TODO(doug): PermissionStorage not implemented");
        Ok(vec![])
    }
}
