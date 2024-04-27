use super::message::async_message_warn;
use crate::slint_generatedAppWindow::{AppWindow, FindEntry as UIFindEntry, Logic, Store};
use crate::{
    config,
    db::{self, ComEntry},
    message_info, message_success,
    util::{http, translator::tr},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Model, VecModel, Weak};
use std::time::Duration;

const FIND_UUID: &str = "find-uuid";
const RSS_ENTRY_URL_CN: &str = "https://heng30.xyz/apisvr/rssbox/rss/list/cn";
const RSS_ENTRY_URL_EN: &str = "https://heng30.xyz/apisvr/rssbox/rss/list/en";
const RSS_VALID_CN: &str = include_str!("../../data/rss-valid-cn.json");
const RSS_VALID_EN: &str = include_str!("../../data/rss-valid-en.json");

#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
pub struct FindEntry {
    pub name: String,
    pub url: String,
}

impl PartialEq for FindEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name || self.url == other.url
    }
}

impl Eq for FindEntry {}

impl From<FindEntry> for UIFindEntry {
    fn from(entry: FindEntry) -> Self {
        UIFindEntry {
            name: entry.name.into(),
            url: entry.url.into(),
        }
    }
}

#[macro_export]
macro_rules! store_find_entrys {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_find_entrys()
            .as_any()
            .downcast_ref::<VecModel<UIFindEntry>>()
            .expect("We know we set a VecModel earlier")
    };
}

#[macro_export]
macro_rules! store_find_entrys_keyword {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_find_entrys_keyword()
            .as_any()
            .downcast_ref::<VecModel<UIFindEntry>>()
            .expect("We know we set a VecModel earlier")
    };
}

fn init_find(ui: &AppWindow) {
    store_find_entrys!(ui).set_vec(vec![]);
    store_find_entrys_keyword!(ui).set_vec(vec![]);

    let rss_configs = super::rss::get_rss_configs(&ui);

    let ui = ui.as_weak();
    tokio::spawn(async move {
        _ = db::entry::new(FIND_UUID).await;

        match db::entry::select_all(FIND_UUID).await {
            Err(e) => log::warn!("{e:?}"),
            Ok(items) => {
                let items = items
                    .into_iter()
                    .filter_map(|item| serde_json::from_str::<FindEntry>(&item.data).ok())
                    .map(|item| item.into())
                    .collect::<Vec<UIFindEntry>>();

                let mut unadd_list = vec![];
                for item in items.into_iter() {
                    if rss_configs
                        .iter()
                        .find(|rss| rss.url.as_str() == item.url.as_str())
                        .is_none()
                    {
                        unadd_list.push(item);
                    }
                }

                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    store_find_entrys!(ui.unwrap()).set_vec(unadd_list);
                });
            }
        }
    });
}

pub fn init(ui: &AppWindow) {
    init_find(&ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_fetch_all_find_entrys(move || {
        let ui = ui_handle.unwrap();
        message_info!(ui, tr("正在刷新..."));

        let ui = ui.as_weak();
        _fetch_all_find_entrys(ui);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_update_find_entrylist(move |keyword| {
            let ui = ui_handle.unwrap();
            let mut keyword_list = vec![];

            for item in ui.global::<Store>().get_find_entrys().iter() {
                if item.name.contains(keyword.as_str()) {
                    keyword_list.push(item);
                }
            }

            store_find_entrys_keyword!(ui).set_vec(keyword_list);
            message_success!(ui, tr("查找完成"));
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_find_entrys_counts(move |_flag| {
        ui_handle
            .unwrap()
            .global::<Store>()
            .get_find_entrys()
            .row_count() as i32
    });
}

async fn _inner_fetch_all_find_entrys() -> Result<Vec<FindEntry>> {
    let url = if config::ui().language == "cn" {
        RSS_ENTRY_URL_CN
    } else {
        RSS_ENTRY_URL_EN
    };

    Ok(http::client(None)?
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json::<Vec<ComEntry>>()
        .await?
        .into_iter()
        .filter_map(|item| serde_json::from_str::<FindEntry>(&item.data).ok())
        .collect::<Vec<_>>())
}

pub fn rss_valid(text: &str) -> Result<Vec<FindEntry>> {
    Ok(serde_json::from_str::<Vec<_>>(text)?)
}

fn _fetch_all_find_entrys(ui: Weak<AppWindow>) {
    tokio::spawn(async move {
        let mut items = match _inner_fetch_all_find_entrys().await {
            Ok(v) => v,
            _ => vec![],
        };

        if items.is_empty() {
            let rss_json = if config::ui().language == "cn" {
                RSS_VALID_CN
            } else {
                RSS_VALID_EN
            };

            match rss_valid(rss_json) {
                Ok(v) => items = v,
                Err(e) => {
                    async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("刷新失败"), tr("原因")),
                    );
                    return;
                }
            }
        }

        _ = db::entry::delete_all(FIND_UUID).await;
        for (index, item) in items.iter().enumerate() {
            if let Ok(data) = serde_json::to_string(item) {
                _ = db::entry::insert(FIND_UUID, &format!("{}", index), &data).await;
            }
        }

        let ui_items = items
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<UIFindEntry>>();

        let ui = ui.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let ui = ui.unwrap();
            store_find_entrys!(ui).set_vec(ui_items);
            message_success!(ui, tr("刷新成功"));
            ui.global::<Store>()
                .set_find_entrys_counts_flag(!ui.global::<Store>().get_find_entrys_counts_flag());
        });
    });
}
