#![windows_subsystem = "windows"]

slint::include_modules!();

#[macro_use]
extern crate lazy_static;

mod config;
mod logic;
mod util;
mod version;

use logic::{clipboard, message};

use anyhow::Result;
use chrono::Local;
use env_logger::fmt::Color as LColor;
use std::io::Write;

fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let mut level_style = buf.style();
            match record.level() {
                log::Level::Warn | log::Level::Error => {
                    level_style.set_color(LColor::Red).set_bold(true)
                }
                _ => level_style.set_color(LColor::Blue).set_bold(true),
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

async fn ui_before() -> Result<()> {
    init_logger();
    config::init();
    Ok(())
}

fn ui_after(ui: &AppWindow) {
    logic::util::init(&ui);
    clipboard::init(&ui);
    message::init(&ui);
}

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() -> Result<()> {
    log::debug!("start...");

    ui_before().await?;
    let ui = AppWindow::new()?;
    ui_after(&ui);
    ui.run().unwrap();

    log::debug!("exit...");
    Ok(())
}

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
async fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    ui_before().await.unwrap();
    let ui = AppWindow::new().unwrap();
    ui_after(&ui);
    ui.run().unwrap();
}
