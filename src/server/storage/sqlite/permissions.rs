use crate::auth::permission::Operation;
use crate::auth::permission::Permission;
use crate::auth::principal::Principal;
use crate::auth::resource_id::ResourceId;
use crate::storage::sqlite::permission_row::PermissionSqliteRow;
use crate::storage::sqlite::schema;
use crate::storage::sqlite::SqliteStorage;
use diesel::insert_into;
use diesel::BoolExpressionMethods;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use futures_util::StreamExt;
use std::fmt::Debug;
use tracing::error;

impl SqliteStorage {
    pub async fn create_permission<T>(&self, permission: Permission<T>) -> Result<(), anyhow::Error>
    where
        Permission<T>: Into<PermissionSqliteRow>,
    {
        let mut connection = self.pool.get()?;
        let permission_row: PermissionSqliteRow = permission.into();
        insert_into(schema::permissions::table)
            .values(permission_row)
            .execute(&mut connection)?;
        Ok(())
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
        let mut connection = self.pool.get()?;
        let rows = schema::permissions::table
            .select(PermissionSqliteRow::as_select())
            .filter(
                schema::permissions::principal_id
                    .eq(principal.id().0)
                    .and(schema::permissions::resource_type.eq(T::resource_type() as i32))
                    .and(
                        schema::permissions::relation
                            .eq_any(operations.iter().map(|op| *op as i32)),
                    ),
            )
            .order_by((
                schema::permissions::project_id,
                schema::permissions::artifact_id,
            ))
            .offset(from as i64)
            .limit(count as i64)
            .load::<PermissionSqliteRow>(&mut connection)?;

        let permissions: Vec<_> = rows
            .into_iter()
            .map(|row| <PermissionSqliteRow as TryInto<Permission<T>>>::try_into(row))
            .filter_map(|result| {
                if let Err(error) = result {
                    error!("Failed to convert permission row: {:?}", error);
                    None
                } else {
                    result.ok()
                }
            })
            .map(|permission| permission.resource_id)
            .collect();
        Ok(permissions)
    }
}
