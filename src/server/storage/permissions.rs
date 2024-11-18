use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::principal::Principal;
use crate::auth::resource_id::ResourceId;
use crate::storage::artifact::Storage;
use crate::storage::sqlite::permission_row::PermissionSqliteRow;
use std::fmt::Debug;

impl Storage {
    pub async fn create_permission<T>(&self, permission: Permission<T>) -> Result<(), anyhow::Error>
    where
        Permission<T>: Into<PermissionSqliteRow>,
    {
        match self {
            Storage::Local(sqlite) => sqlite.create_permission(permission).await,
        }
    }

    pub async fn get_resources<T>(
        &self,
        principal: &Principal,
        operations: Vec<Operation>,
        from: usize,
        count: usize,
    ) -> Result<Vec<T>, anyhow::Error>
    where
        T: ResourceId,
        Permission<T>: TryFrom<PermissionSqliteRow>,
        <Permission<T> as TryFrom<PermissionSqliteRow>>::Error: Debug,
    {
        match self {
            Storage::Local(sqlite) => {
                sqlite
                    .get_resources(principal, operations, from, count)
                    .await
            }
        }
    }
}
