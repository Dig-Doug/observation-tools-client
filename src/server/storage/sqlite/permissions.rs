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
use tracing::warn;

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
        operation: Operation,
        from: usize,
        count: usize,
    ) -> Result<Vec<T>, anyhow::Error>
    where
        T: ResourceId,
    {
        let mut connection = self.pool.get()?;
        let permissions = schema::permissions::table
            .select(PermissionSqliteRow::as_select())
            .filter(
                schema::permissions::principal_id
                    .eq(principal.id().0)
                    .and(schema::permissions::resource_type.eq(T::resource_type() as i32))
                    .and(schema::permissions::relation.eq(operation as i32)),
            )
            .order_by((
                schema::permissions::project_id,
                schema::permissions::artifact_id,
            ))
            .load::<PermissionSqliteRow>(&mut connection)?;

        warn!("TODO(doug): PermissionStorage not implemented");
        Ok(vec![])
    }
}
