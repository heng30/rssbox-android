#![windows_subsystem = "windows"]

slint::include_modules!();

#[macro_use]
extern crate lazy_static;

use chrono::Utc;
use slint::{Timer, TimerMode};
use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};

mod config;
mod db;
pub mod logic;
pub mod util;
mod version;

static SYNC_TIMESTAMP_CACHE: AtomicI64 = AtomicI64::new(0);

#[cfg(not(target_os = "android"))]
pub fn init_logger() {
    use chrono::Local;
    use env_logger::fmt::Color;
    use std::io::Write;

    env_logger::builder()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let mut level_style = buf.style();
            match record.level() {
                log::Level::Warn | log::Level::Error => {
                    level_style.set_color(Color::Red).set_bold(true)
                }
                _ => level_style.set_color(Color::Blue).set_bold(true),
            };

            writeln!(
                buf,
                "[{} {} {} {}] {}",
                ts,
                level_style.value(record.level()),
                record
                    .file()
                    .unwrap_or("None")
                    .split('/')
                    .last()
                    .unwrap_or("None"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}

#[cfg(target_os = "android")]
fn init_logger() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(
                android_logger::FilterBuilder::new()
                    .filter_level(log::LevelFilter::Debug)
                    .build(),
            ),
    );
}

async fn ui_before() {
    init_logger();
    config::init();
    db::init(config::db_path().to_str().expect("invalid db path")).await;
}

fn ui_after(ui: &AppWindow) {
    logic::init(ui);
}

fn sync_rss_timer(ui: &AppWindow) -> Timer {
    let ui_handle = ui.as_weak();
    SYNC_TIMESTAMP_CACHE.store(Utc::now().timestamp(), Ordering::SeqCst);

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(10), move || {
        let config = config::sync();
        let now = Utc::now().timestamp();

        if config.is_auto_sync {
            let sync_interval = i64::max(config.sync_interval as i64, 1_i64) * 60;
            if SYNC_TIMESTAMP_CACHE.load(Ordering::SeqCst) + sync_interval <= now {
                SYNC_TIMESTAMP_CACHE.store(now, Ordering::SeqCst);

                let ui = ui_handle.unwrap();
                ui.global::<Logic>().invoke_sync_rss_all();
            }
        } else {
            SYNC_TIMESTAMP_CACHE.store(now, Ordering::SeqCst);
        }
    });
    timer
}

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
async fn android_main(app: slint::android::AndroidApp) {
    log::debug!("start...");

    slint::android::init(app).unwrap();
    ui_before().await;
    let ui = AppWindow::new().unwrap();
    ui_after(&ui);

    let _timer = sync_rss_timer(&ui);
    ui.run().unwrap();

    log::debug!("exit...");
}

#[cfg(not(target_os = "android"))]
pub async fn desktop_main() {
    log::debug!("start...");

    ui_before().await;
    let ui = AppWindow::new().unwrap();
    ui_after(&ui);

    let _timer = sync_rss_timer(&ui);
    ui.run().unwrap();

    log::debug!("exit...");
}
