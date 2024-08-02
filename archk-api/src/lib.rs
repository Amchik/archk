use sqlx::SqlitePool;

pub mod app;
pub mod roles;
pub mod v1;

pub async fn apply_migrations(db: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(db).await
}
