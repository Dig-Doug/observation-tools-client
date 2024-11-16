use crate::auth::permission::Permission;
use diesel::Insertable;
use diesel::Queryable;
use diesel::Selectable;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::storage::sqlite::schema::permissions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PermissionSqliteRow {
    pub principal_id: String,
    pub resource_id: String,
    pub relation: String,
}

impl<T> TryFrom<Permission<T>> for PermissionSqliteRow {
    type Error = anyhow::Error;

    fn try_from(value: Permission<T>) -> Result<Self, Self::Error> {
        todo!("Impl")
    }
}

impl<T> TryFrom<PermissionSqliteRow> for Permission<T> {
    type Error = anyhow::Error;

    fn try_from(value: PermissionSqliteRow) -> Result<Self, Self::Error> {
        todo!("Impl")
    }
}
