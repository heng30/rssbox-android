use super::{pool, ComEntry};
use crate::slint_generatedAppWindow::RssEntry as UIRssEntry;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct RssEntry {
    pub suuid: String,
    pub uuid: String,
    pub url: String,
    pub title: String,
    pub pub_date: String,
    pub tags: String,
    pub author: String,
    pub summary: String,
    pub is_read: bool,
}

impl From<UIRssEntry> for RssEntry {
    fn from(entry: UIRssEntry) -> Self {
        RssEntry {
            suuid: entry.suuid.clone().into(),
            uuid: entry.uuid.clone().into(),
            url: entry.url.clone().into(),
            title: entry.title.clone().into(),
            tags: entry.tags.clone().into(),
            pub_date: entry.pub_date.clone().into(),
            author: entry.author.clone().into(),
            summary: entry.summary.clone().into(),
            is_read: entry.is_read,
        }
    }
}

impl From<RssEntry> for UIRssEntry {
    fn from(entry: RssEntry) -> Self {
        UIRssEntry {
            suuid: entry.suuid.into(),
            uuid: entry.uuid.into(),
            url: entry.url.into(),
            pub_date: entry.pub_date.into(),
            title: entry.title.into(),
            tags: entry.tags.into(),
            author: entry.author.into(),
            summary: entry.summary.into(),
            is_read: entry.is_read,
        }
    }
}

fn table_name(suuid: &str) -> String {
    "entry_".to_string() + &suuid.replace('-', "_")
}

pub async fn new(suuid: &str) -> Result<()> {
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {} (
             id INTEGER PRIMARY KEY,
             uuid TEXT NOT NULL UNIQUE,
             data TEXT NOT NULL
             )",
        table_name(suuid)
    ))
    .execute(&pool())
    .await?;

    Ok(())
}

pub async fn delete(suuid: &str, uuid: &str) -> Result<()> {
    sqlx::query(&format!("DELETE FROM {} WHERE uuid=?", table_name(suuid)))
        .bind(uuid)
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn delete_all(suuid: &str) -> Result<()> {
    sqlx::query(&format!("DELETE FROM {}", table_name(suuid)))
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn insert(suuid: &str, uuid: &str, data: &str) -> Result<()> {
    sqlx::query(&format!(
        "INSERT INTO {} (uuid, data) VALUES (?, ?)",
        table_name(suuid)
    ))
    .bind(uuid)
    .bind(data)
    .execute(&pool())
    .await?;
    Ok(())
}

pub async fn update(suuid: &str, uuid: &str, data: &str) -> Result<()> {
    sqlx::query(&format!(
        "UPDATE {} SET data=? WHERE uuid=?",
        table_name(suuid)
    ))
    .bind(data)
    .bind(uuid)
    .execute(&pool())
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn select(suuid: &str, uuid: &str) -> Result<ComEntry> {
    Ok(
        sqlx::query_as::<_, ComEntry>(&format!("SELECT * FROM {} WHERE uuid=?", table_name(suuid)))
            .bind(uuid)
            .fetch_one(&pool())
            .await?,
    )
}

pub async fn select_all(suuid: &str) -> Result<Vec<ComEntry>> {
    Ok(
        sqlx::query_as::<_, ComEntry>(&format!("SELECT * FROM {}", table_name(suuid)))
            .fetch_all(&pool())
            .await?,
    )
}

pub async fn drop_table(suuid: &str) -> Result<()> {
    super::drop_table(&table_name(suuid)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::sync::Mutex;

    static MTX: Mutex<()> = Mutex::new(());
    const DB_PATH: &str = "/tmp/rssbox-entry-test.db";

    #[tokio::test]
    async fn test_table_new() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_all() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_one() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;

        delete_all("suuid-1").await?;
        insert("suuid-1", "uuid-1", "data-1").await?;
        delete("suuid-1", "uuid-1").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;

        insert("suuid-1", "uuid-1", "data-1").await?;
        insert("suuid-1", "uuid-2", "data-2").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_update() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;

        insert("suuid-1", "uuid-1", "data-1").await?;
        update("suuid-1", "uuid-1", "data-1-1").await?;

        assert_eq!(
            select("suuid-1", "uuid-1").await?.data,
            "data-1-1".to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_select_one() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;

        assert!(select("suuid-1", "uuid-1").await.is_err());

        insert("suuid-1", "uuid-1", "data-1").await?;
        let item = select("suuid-1", "uuid-1").await?;
        assert_eq!(item.uuid, "uuid-1");
        assert_eq!(item.data, "data-1");
        Ok(())
    }

    #[tokio::test]
    async fn test_select_all() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;

        insert("suuid-1", "uuid-1", "data-1").await?;
        insert("suuid-1", "uuid-2", "data-2").await?;

        let v = select_all("suuid-1").await?;
        assert_eq!(v[0].uuid, "uuid-1");
        assert_eq!(v[0].data, "data-1");
        assert_eq!(v[1].uuid, "uuid-2");
        assert_eq!(v[1].data, "data-2");
        Ok(())
    }

    #[tokio::test]
    async fn test_drop_table() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new("suuid-1").await?;
        delete_all("suuid-1").await?;
        insert("suuid-1", "uuid-1", "data-1").await?;

        assert!(drop_table("suuid-0").await.is_err());
        assert!(drop_table("suuid-1").await.is_ok());
        Ok(())
    }
}
