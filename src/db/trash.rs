use super::pool;
use anyhow::Result;
use sqlx::Row;

pub async fn new() -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS trash (
             id INTEGER PRIMARY KEY,
             md5 TEXT NOT NULL UNIQUE
             )",
    )
    .execute(&pool())
    .await?;

    Ok(())
}

pub async fn insert(md5: &str) -> Result<()> {
    sqlx::query("INSERT INTO trash (md5) VALUES (?)")
        .bind(md5)
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn delete_all() -> Result<()> {
    sqlx::query("DELETE FROM trash").execute(&pool()).await?;
    Ok(())
}

pub async fn select(md5: &str) -> Result<String> {
    let row = sqlx::query("SELECT * FROM trash WHERE md5=?")
        .bind(md5)
        .fetch_one(&pool())
        .await?;

    Ok(row.try_get("md5")?)
}

pub async fn is_exist(md5: &str) -> Result<()> {
    select(md5).await?;
    Ok(())
}

pub async fn row_count() -> Result<i32> {
    let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM trash")
        .fetch_one(&pool())
        .await?;

    Ok(count.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::sync::Mutex;

    static MTX: Mutex<()> = Mutex::new(());
    const DB_PATH: &str = "/tmp/rssbox-trash-test.db";

    #[tokio::test]
    async fn test_table_new() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await
    }

    #[tokio::test]
    async fn test_delete_all() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await
    }

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        insert("md5-1").await?;
        insert("md5-2").await
    }

    #[tokio::test]
    async fn test_select_one() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        assert!(select("md5-1").await.is_err());

        insert("md5-1").await?;
        assert_eq!(select("md5-1").await?, "md5-1");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_exist() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        insert("md5-1").await?;

        assert!(is_exist("md5-0").await.is_err());
        assert!(is_exist("md5-1").await.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_row_count() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        assert_eq!(row_count().await.unwrap(), 0);

        insert("md5-1").await?;
        assert_eq!(row_count().await.unwrap(), 1);
        Ok(())
    }
}
