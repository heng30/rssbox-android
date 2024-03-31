use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,

    #[serde(skip)]
    pub db_path: PathBuf,

    #[serde(skip)]
    pub cache_dir: PathBuf,

    #[serde(default = "appid_default")]
    pub appid: String,

    #[serde(skip)]
    pub is_first_run: bool,

    pub ui: UI,

    pub reading: Reading,

    pub sync: Sync,

    pub proxy: Proxy,
}

pub fn appid_default() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UI {
    pub font_size: u32,
    pub font_family: String,
    pub language: String,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            font_size: 16,
            font_family: "SourceHanSerifCN".to_string(),
            language: "cn".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proxy {
    pub http_url: String,
    pub http_port: u16,
    pub socks5_url: String,
    pub socks5_port: u16,
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            http_url: "127.0.0.1".to_string(),
            http_port: 3128,
            socks5_url: "127.0.0.1".to_string(),
            socks5_port: 1080,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sync {
    pub sync_interval: i64,
    pub sync_timeout: i64,
    pub is_auto_sync: bool,
    pub is_start_sync: bool,
}

impl Default for Sync {
    fn default() -> Self {
        Self {
            sync_interval: 60,
            sync_timeout: 15,
            is_auto_sync: true,
            is_start_sync: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reading {
    pub browser: String,
    pub is_delete_after_reading: bool,
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            browser: "Default".to_string(),
            is_delete_after_reading: false,
        }
    }
}
