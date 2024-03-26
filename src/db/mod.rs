use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::{
    sqlite::{Sqlite, SqlitePoolOptions},
    Pool,
};
use std::sync::Mutex;

pub mod entry;
pub mod rss;
pub mod trash;

const MAX_CONNECTIONS: u32 = 3;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct ComEntry {
    pub uuid: String,
    pub data: String,
}

lazy_static! {
    static ref POOL: Mutex<Option<Pool<Sqlite>>> = Mutex::new(None);
}

fn pool() -> Pool<Sqlite> {
    POOL.lock().unwrap().clone().unwrap()
}

async fn create_db(db_path: &str) -> Result<(), sqlx::Error> {
    Sqlite::create_database(db_path).await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(&format!("sqlite:{}", db_path))
        .await?;

    *POOL.lock().unwrap() = Some(pool);

    Ok(())
}

pub async fn init(db_path: &str) {
    create_db(db_path).await.expect("create db");
    rss::new().await.expect("rss table failed");
    trash::new().await.expect("trash table failed");
}

#[allow(dead_code)]
pub async fn is_table_exist(table_name: &str) -> Result<()> {
    sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
        .bind(table_name)
        .fetch_one(&pool())
        .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_table(table_name: &str) -> Result<()> {
    sqlx::query(&format!("DROP TABLE {}", table_name))
        .execute(&pool())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static MTX: Mutex<()> = Mutex::new(());
    const DB_PATH: &str = "/tmp/rssbox-test.db";

    #[tokio::test]
    async fn test_db_is_table_exist() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        init(DB_PATH).await;
        trash::new().await?;
        assert!(is_table_exist("hello").await.is_err());
        assert!(is_table_exist("trash").await.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_db_drop_table() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        init(DB_PATH).await;
        trash::new().await?;
        assert!(drop_table("hello").await.is_err());
        assert!(drop_table("trash").await.is_ok());
        Ok(())
    }
}
