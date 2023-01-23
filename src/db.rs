use sqlx::SqlitePool;

const CREATE_DB_SQL: &str = include_str!("../migrations/01-create-db.sql");
const CREATE_INDEXES_SQL: &str = include_str!("../migrations/02-create-indexes.sql");
const DELETE_INDEXES_SQL: &str = include_str!("../migrations/99-delete-indexes.sql");

pub async fn create_db(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(CREATE_DB_SQL).execute(db).await?;
    Ok(())
}

pub async fn create_indexes(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(CREATE_INDEXES_SQL).execute(db).await?;
    Ok(())
}

pub async fn delete_indexes(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(DELETE_INDEXES_SQL).execute(db).await?;
    Ok(())
}
