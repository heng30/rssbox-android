use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::{
    config,
    db::{self, entry::RssEntry, rss},
    util::{crypto::md5_hex, translator::tr},
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use webbrowser;

pub async fn get_from_db(suuid: &str) -> Vec<UIRssEntry> {
    match db::entry::select_all(suuid).await {
        Ok(items) => items
            .into_iter()
            .rev()
            .filter_map(|item| serde_json::from_str::<RssEntry>(&item.data).ok())
            .map(|item| item.into())
            .collect(),
        Err(e) => {
            log::warn!("{:?}", e);
            vec![]
        }
    }
}

pub fn init(ui: &AppWindow) {}
