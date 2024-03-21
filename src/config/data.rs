use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,

    #[serde(skip)]
    pub db_path: PathBuf,

    #[serde(skip)]
    pub cache_dir: PathBuf,

    pub ui: UI,

    pub socks5: Socks5,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: PathBuf::default(),
            db_path: PathBuf::default(),
            cache_dir: PathBuf::default(),
            ui: UI::default(),
            socks5: Socks5::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UI {
    pub font_size: u32,
    pub font_family: String,
    pub win_width: u32,
    pub win_height: u32,
    pub language: String,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            font_size: 18,
            font_family: "SourceHanSerifCN".to_string(),
            win_width: 600,
            win_height: 800,
            language: "cn".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Socks5 {
    pub enabled: bool,
    pub url: String,
    pub port: u16,
}

impl Default for Socks5 {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "127.0.0.1".to_string(),
            port: 1080,
        }
    }
}
