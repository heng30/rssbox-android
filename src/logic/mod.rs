use crate::slint_generatedAppWindow::AppWindow;

mod about;
mod cache;
mod clipboard;
mod entry;
mod message;
mod ok_cancel_dialog;
mod rss;
mod setting;
mod util;

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
}
