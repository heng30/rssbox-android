use super::pool;
use anyhow::Result;
use sqlx::Row;

pub async fn new() -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS blacklist (
             id INTEGER PRIMARY KEY,
             md5 TEXT NOT NULL UNIQUE
             )",
    )
    .execute(&pool())
    .await?;

    Ok(())
}

pub async fn insert(md5: &str) -> Result<()> {
    sqlx::query("INSERT INTO blacklist (md5) VALUES (?)")
        .bind(md5)
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn delete(md5: &str) -> Result<()> {
    sqlx::query("DELETE FROM blacklist WHERE md5=?")
        .bind(md5)
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn select(md5: &str) -> Result<String> {
    let row = sqlx::query("SELECT * FROM blacklist WHERE md5=?")
        .bind(md5)
        .fetch_one(&pool())
        .await?;

    Ok(row.try_get("md5")?)
}

pub async fn is_exist(md5: &str) -> Result<()> {
    select(md5).await?;
    Ok(())
}
