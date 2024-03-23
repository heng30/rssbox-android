use super::data::{self, Config};
use anyhow::Result;
use log::debug;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    pub static ref CONFIG: Mutex<RefCell<Config>> = Mutex::new(RefCell::new(Config::default()));
}

#[cfg(not(target_os = "android"))]
use platform_dirs::AppDirs;

#[cfg(target_os = "android")]
pub struct AppDirs {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

#[cfg(target_os = "android")]
impl AppDirs {
    pub fn new(name: Option<&str>, _: bool) -> Option<Self> {
        let root_dir = "/data/data";
        let name = name.unwrap();

        Some(Self {
            config_dir: PathBuf::from(&format!("{root_dir}/{name}/config")),
            data_dir: PathBuf::from(&format!("{root_dir}/{name}/data")),
        })
    }
}

pub fn init() {
    if let Err(e) = CONFIG.lock().unwrap().borrow_mut().init() {
        panic!("{:?}", e);
    }
}

pub fn ui() -> data::UI {
    CONFIG.lock().unwrap().borrow().ui.clone()
}

pub fn proxy() -> data::Proxy {
    CONFIG.lock().unwrap().borrow().proxy.clone()
}

pub fn db_path() -> PathBuf {
    let conf = CONFIG.lock().unwrap();
    let conf = conf.borrow();

    conf.db_path.clone()
}

pub fn cache_dir() -> PathBuf {
    let conf = CONFIG.lock().unwrap();
    let conf = conf.borrow();

    conf.cache_dir.clone()
}

pub fn save(conf: data::Config) -> Result<()> {
    let config = CONFIG.lock().unwrap();
    let mut config = config.borrow_mut();
    *config = conf;
    config.save()
}

impl Config {
    pub fn init(&mut self) -> Result<()> {
        let app_name = if cfg!(not(target_os = "android")) {
            "rssbox-android"
        } else {
            "xyz.heng30.rssbox"
        };

        let app_dirs = AppDirs::new(Some(app_name), true).unwrap();
        self.init_config(&app_dirs)?;
        self.load()?;
        debug!("{:?}", self);
        Ok(())
    }

    fn init_config(&mut self, app_dirs: &AppDirs) -> Result<()> {
        self.db_path = app_dirs.data_dir.join("rssbox.db");
        self.config_path = app_dirs.config_dir.join("rssbox.toml");
        self.cache_dir = app_dirs.data_dir.join("cache");

        fs::create_dir_all(&app_dirs.data_dir)?;
        fs::create_dir_all(&app_dirs.config_dir)?;
        fs::create_dir_all(&self.cache_dir)?;

        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        match fs::read_to_string(&self.config_path) {
            Ok(text) => match toml::from_str::<Config>(&text) {
                Ok(c) => {
                    self.ui = c.ui;
                    self.proxy = c.proxy;
                    self.sync = c.sync;
                    Ok(())
                }
                Err(e) => Err(e.into()),
            },

            Err(_) => match toml::to_string_pretty(self) {
                Ok(text) => Ok(fs::write(&self.config_path, text)?),
                Err(e) => Err(e.into()),
            },
        }
    }

    pub fn save(&self) -> Result<()> {
        match toml::to_string_pretty(self) {
            Ok(text) => Ok(fs::write(&self.config_path, text)?),
            Err(e) => Err(e.into()),
        }
    }
}
