use crate::types::Update;

use sqlx::SqlitePool;

#[allow(dead_code)]
pub enum UpdateType {
    Daily,
    Weekly,
    Any,
}

pub async fn get_last_update(
    db: &SqlitePool,
    update_type: UpdateType,
) -> Result<Option<Update>, sqlx::Error> {
    let query_str = match update_type {
        UpdateType::Daily => "SELECT * FROM updates WHERE daily = 1 ORDER BY id DESC LIMIT 1",
        UpdateType::Weekly => "SELECT * FROM updates WHERE weekly = 1 ORDER BY id DESC LIMIT 1",
        UpdateType::Any => "SELECT * FROM updates ORDER BY id DESC LIMIT 1",
    };
    let update = sqlx::query_as::<_, Update>(query_str)
        .fetch_optional(db)
        .await?;
    Ok(update)
}

pub async fn insert_update(db: &SqlitePool, update: &Update) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO updates (daily, weekly, date) VALUES (?, ?, ?)")
        .bind(update.daily)
        .bind(update.weekly)
        .bind(update.date)
        .execute(db)
        .await?;
    Ok(())
}
