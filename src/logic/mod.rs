use crate::slint_generatedAppWindow::AppWindow;

mod about;
mod cache;
mod clipboard;
mod entry;
mod message;
mod rss;
mod util;

pub fn init(ui: &AppWindow) {
    util::init(&ui);
    clipboard::init(&ui);
    message::init(&ui);
    cache::init(&ui);
    about::init(&ui);
    rss::init(&ui);
    entry::init(&ui);
}
