use crate::slint_generatedAppWindow::AppWindow;

mod about;
mod cache;
mod clipboard;
mod entry;
mod message;
mod rss;
mod util;
mod ok_cancel_dialog;

pub fn init(ui: &AppWindow) {
    util::init(&ui);
    clipboard::init(&ui);
    message::init(&ui);
    ok_cancel_dialog::init(&ui);
    cache::init(&ui);
    about::init(&ui);

    // don't adjust functions order
    entry::init(&ui);
    rss::init(&ui);
}
