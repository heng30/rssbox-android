use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::{
    config,
    db::{self, entry::RssEntry, rss},
    util::{crypto::md5_hex, translator::tr},
};
use anyhow::Result;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use webbrowser;

const FAVORITE_UUID: &str = "favorite-uuid";

#[macro_export]
macro_rules! store_rss_entrys {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_rss_entrys()
            .as_any()
            .downcast_ref::<VecModel<UIRssEntry>>()
            .expect("We know we set a VecModel earlier")
    };
}

#[macro_export]
macro_rules! store_favorite_entrys {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_rss_favorite_entrys()
            .as_any()
            .downcast_ref::<VecModel<UIRssEntry>>()
            .expect("We know we set a VecModel earlier")
    };
}

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

fn init_favorite(ui: Weak<AppWindow>) {
    tokio::spawn(async move {
        db::entry::new(FAVORITE_UUID).await.unwrap();
        let entry_list = get_from_db(FAVORITE_UUID).await;

        let _ = slint::invoke_from_event_loop(move || {
            store_favorite_entrys!(ui.unwrap()).set_vec(entry_list);
        });
    });
}

pub fn init(ui: &AppWindow) {
    init_favorite(ui.as_weak());

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_entry(move |suuid, uuid| {
        let ui = ui_handle.unwrap();

        for (index, entry) in ui.global::<Store>().get_rss_entrys().iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            store_rss_entrys!(ui).remove(index);
            _remove_entry(ui.as_weak(), suuid, uuid);
            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_entrys(move |suuid| {
        if suuid.is_empty() {
            return;
        }

        let ui = ui_handle.unwrap();
        store_rss_entrys!(ui).set_vec(vec![]);
        _remove_all_entrys(ui.as_weak(), suuid);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_favorite_entry(move |uuid| {
        let ui = ui_handle.unwrap();
        for (index, entry) in ui
            .global::<Store>()
            .get_rss_favorite_entrys()
            .iter()
            .enumerate()
        {
            if uuid != entry.uuid {
                continue;
            }

            store_favorite_entrys!(ui).remove(index);
            break;
        }

        _remove_favorite_entry(ui.as_weak(), uuid);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_favorite_entrys(move || {
        let ui = ui_handle.unwrap();
        store_favorite_entrys!(ui).set_vec(vec![]);
        _remove_all_favorite_entrys(ui.as_weak());
    });


    // callback favorite-entry(string, string); // suuid, uuid
    // callback set-entry-read(string, string); // suuid, uuid
}

fn _remove_entry(ui: Weak<AppWindow>, suuid: SharedString, uuid: SharedString) {
    tokio::spawn(async move {
        match db::entry::delete(suuid.as_str(), uuid.as_str()).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("删除成功")),
        }
    });
}

fn _remove_all_entrys(ui: Weak<AppWindow>, suuid: SharedString) {
    tokio::spawn(async move {
        match db::entry::delete_all(suuid.as_str()).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("删除成功")),
        }
    });
}

fn _remove_favorite_entry(ui: Weak<AppWindow>, uuid: SharedString) {
    tokio::spawn(async move {
        match db::entry::delete(FAVORITE_UUID, uuid.as_str()).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("删除成功")),
        }
    });
}

fn _remove_all_favorite_entrys(ui: Weak<AppWindow>) {
    tokio::spawn(async move {
        match db::entry::delete_all(FAVORITE_UUID).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("删除成功")),
        }
    });
}
