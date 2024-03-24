use super::{pool, ComEntry};
use crate::slint_generatedAppWindow::RssConfig as UIRssConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

fn feed_format_default() -> String {
    "AUTO".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RssConfig {
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub icon_index: i32,
    pub use_http_proxy: bool,
    pub use_socks5_proxy: bool,
    pub is_favorite: bool,
    pub update_time: String,

    #[serde(default = "feed_format_default")]
    pub feed_format: String,
}

impl From<UIRssConfig> for RssConfig {
    fn from(conf: UIRssConfig) -> Self {
        RssConfig {
            uuid: conf.uuid.into(),
            name: conf.name.into(),
            url: conf.url.into(),
            icon_index: conf.icon_index,
            use_http_proxy: conf.use_http_proxy,
            use_socks5_proxy: conf.use_socks5_proxy,
            is_favorite: conf.is_favorite,
            update_time: conf.update_time.into(),
            feed_format: conf.feed_format.into(),
        }
    }
}

impl From<RssConfig> for UIRssConfig {
    fn from(conf: RssConfig) -> Self {
        UIRssConfig {
            uuid: conf.uuid.into(),
            name: conf.name.into(),
            url: conf.url.into(),
            icon_index: conf.icon_index,
            use_http_proxy: conf.use_http_proxy,
            use_socks5_proxy: conf.use_socks5_proxy,
            is_favorite: conf.is_favorite,
            update_time: conf.update_time.into(),
            feed_format: conf.feed_format.into(),
            ..Default::default()
        }
    }
}

pub async fn new() -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rss (
             id INTEGER PRIMARY KEY,
             uuid TEXT NOT NULL UNIQUE,
             data TEXT NOT NULL
             )",
    )
    .execute(&pool())
    .await?;

    Ok(())
}

pub async fn delete(uuid: &str) -> Result<()> {
    sqlx::query("DELETE FROM rss WHERE uuid=?")
        .bind(uuid)
        .execute(&pool())
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_all() -> Result<()> {
    sqlx::query("DELETE FROM rss").execute(&pool()).await?;
    Ok(())
}

pub async fn insert(uuid: &str, data: &str) -> Result<()> {
    sqlx::query("INSERT INTO rss (uuid, data) VALUES (?, ?)")
        .bind(uuid)
        .bind(data)
        .execute(&pool())
        .await?;
    Ok(())
}

pub async fn update(uuid: &str, data: &str) -> Result<()> {
    sqlx::query("UPDATE rss SET data=? WHERE uuid=?")
        .bind(data)
        .bind(uuid)
        .execute(&pool())
        .await?;

    Ok(())
}

pub async fn select(uuid: &str) -> Result<ComEntry> {
    Ok(
        sqlx::query_as::<_, ComEntry>("SELECT * FROM rss WHERE uuid=?")
            .bind(uuid)
            .fetch_one(&pool())
            .await?,
    )
}

pub async fn select_all() -> Result<Vec<ComEntry>> {
    Ok(sqlx::query_as::<_, ComEntry>("SELECT * FROM rss")
        .fetch_all(&pool())
        .await?)
}

pub async fn is_exist(uuid: &str) -> Result<()> {
    select(uuid).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::sync::Mutex;

    static MTX: Mutex<()> = Mutex::new(());
    const DB_PATH: &str = "/tmp/rssbox-rss-test.db";

    #[tokio::test]
    async fn test_table_new() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_all() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_one() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;

        delete_all().await?;
        insert("uuid-1", "data-1").await?;
        delete("uuid-1").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;

        insert("uuid-1", "data-1").await?;
        insert("uuid-2", "data-2").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_update() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;

        insert("uuid-1", "data-1").await?;
        update("uuid-1", "data-1-1").await?;

        assert_eq!(select("uuid-1").await?.data, "data-1-1".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_select_one() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;

        assert!(select("uuid-1").await.is_err());

        insert("uuid-1", "data-1").await?;
        let item = select("uuid-1").await?;
        assert_eq!(item.uuid, "uuid-1");
        assert_eq!(item.data, "data-1");
        Ok(())
    }

    #[tokio::test]
    async fn test_select_all() -> Result<()> {
        let _mtx = MTX.lock().unwrap();

        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;

        insert("uuid-1", "data-1").await?;
        insert("uuid-2", "data-2").await?;

        let v = select_all().await?;
        assert_eq!(v[0].uuid, "uuid-1");
        assert_eq!(v[0].data, "data-1");
        assert_eq!(v[1].uuid, "uuid-2");
        assert_eq!(v[1].data, "data-2");
        Ok(())
    }

    #[tokio::test]
    async fn test_is_exist() -> Result<()> {
        let _mtx = MTX.lock().unwrap();
        db::init(DB_PATH).await;
        new().await?;
        delete_all().await?;
        insert("uuid-1", "data-1").await?;

        assert!(is_exist("uuid-0").await.is_err());
        assert!(is_exist("uuid-1").await.is_ok());
        Ok(())
    }
}
