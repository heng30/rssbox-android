use crate::config;
use crate::slint_generatedAppWindow::AppWindow;
use serde::{Deserialize, Serialize};

mod about;
mod cache;
mod clipboard;
mod entry;
mod find;
mod message;
mod ok_cancel_dialog;
mod rss;
mod setting;
mod util;

pub use find::{top_rss_list_cn, FindEntry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqData {
    appid: String,
    r#type: String,
    data: String,
}

impl Default for ReqData {
    fn default() -> Self {
        Self {
            appid: config::appid(),
            r#type: Default::default(),
            data: Default::default(),
        }
    }
}

pub fn init(ui: &AppWindow) {
    util::init(&ui);
    clipboard::init(&ui);
    message::init(&ui);
    ok_cancel_dialog::init(&ui);
    cache::init(&ui);
    about::init(&ui);
    setting::init(&ui);

    // don't adjust functions order
    entry::init(&ui);
    rss::init(&ui);
    find::init(&ui);
}
