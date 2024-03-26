use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{AppWindow, Logic, RssEntry as UIRssEntry, Store};
use crate::{
    db::{self, entry::RssEntry},
    message_info,
    util::{crypto::md5_hex, translator::tr},
};
use anyhow::Result;
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};

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

            if !entry.is_read {
                super::rss::decease_unread_counts(&ui, &suuid);
            }

            store_rss_entrys!(ui).remove(index);
            _remove_entry(ui.as_weak(), suuid, uuid, entry.url);
            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_entrys(move |suuid| {
        if suuid.is_empty() {
            return;
        }

        let ui = ui_handle.unwrap();

        super::rss::reset_unread_counts(&ui, &suuid);

        let urls = store_rss_entrys!(ui)
            .iter()
            .map(|item| item.url.clone())
            .collect::<Vec<_>>();

        store_rss_entrys!(ui).set_vec(vec![]);
        _remove_all_entrys(ui.as_weak(), suuid, urls);
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

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_favorite_entry(move |_suuid, uuid| {
        let ui = ui_handle.unwrap();

        for entry in ui.global::<Store>().get_rss_entrys().iter() {
            if entry.uuid != uuid {
                continue;
            }

            for favorite_entry in ui.global::<Store>().get_rss_favorite_entrys().iter() {
                if favorite_entry.uuid != uuid {
                    continue;
                }
                message_info!(ui, tr("已经收藏"));
                return;
            }

            store_favorite_entrys!(ui).insert(0, entry.clone());
            _favorite_entry(ui.as_weak(), entry.into());

            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_entry_read(move |suuid, uuid| {
        let ui = ui_handle.unwrap();

        super::rss::decease_unread_counts(&ui, &suuid);

        for (index, mut entry) in ui.global::<Store>().get_rss_entrys().iter().enumerate() {
            if entry.uuid != uuid {
                continue;
            }

            entry.is_read = true;
            store_rss_entrys!(ui).set_row_data(index, entry.clone());
            _set_entry_read(ui.as_weak(), suuid, entry.into());

            return;
        }
    });
}

fn _remove_entry(ui: Weak<AppWindow>, suuid: SharedString, uuid: SharedString, url: SharedString) {
    tokio::spawn(async move {
        _ = db::trash::insert(&md5_hex(&url)).await;

        match db::entry::delete(suuid.as_str(), uuid.as_str()).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("删除成功")),
        }
    });
}

fn _remove_all_entrys(ui: Weak<AppWindow>, suuid: SharedString, urls: Vec<SharedString>) {
    tokio::spawn(async move {
        for url in urls.into_iter() {
            _ = db::trash::insert(&md5_hex(&url)).await;
        }

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

async fn _inner_favorite_entry(entry: RssEntry) -> Result<()> {
    let data = serde_json::to_string(&entry)?;
    db::entry::insert(FAVORITE_UUID, entry.uuid.as_str(), &data).await?;
    Ok(())
}

fn _favorite_entry(ui: Weak<AppWindow>, entry: RssEntry) {
    tokio::spawn(async move {
        match _inner_favorite_entry(entry).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("收藏失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("收藏成功")),
        }
    });
}

async fn _inner_set_entry_read(suuid: &str, entry: RssEntry) -> Result<()> {
    let data = serde_json::to_string(&entry)?;
    db::entry::update(suuid, entry.uuid.as_str(), &data).await?;
    Ok(())
}

fn _set_entry_read(ui: Weak<AppWindow>, suuid: SharedString, entry: RssEntry) {
    tokio::spawn(async move {
        match _inner_set_entry_read(suuid.as_str(), entry).await {
            Err(e) => async_message_warn(ui, format!("{}. {}: {e:?}", tr("保存失败"), tr("原因"))),
            _ => (),
        }
    });
}

async fn update_new_entry(suuid: &str, entry: RssEntry) -> Result<()> {
    let data = serde_json::to_string(&entry)?;
    db::entry::insert(suuid, entry.uuid.as_str(), &data).await?;
    Ok(())
}

pub fn update_new_entrys(ui: &AppWindow, suuid: &str, entrys: Vec<RssEntry>) {
    for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
        if rss.uuid != suuid {
            continue;
        }

        let mut unfound_list = vec![];
        for entry in entrys.into_iter() {
            if rss.entry.iter().find(|v| v.url == entry.url).is_none() {
                unfound_list.push(entry);
            }
        }

        rss.unread_counts += unfound_list.len() as i32;

        for mut item in unfound_list.into_iter() {
            item.suuid = suuid.into();

            rss.entry
                .as_any()
                .downcast_ref::<VecModel<UIRssEntry>>()
                .expect("We know we set a VecModel earlier")
                .insert(0, item.clone().into());

            let suuid = suuid.to_string();
            tokio::spawn(async move {
                if let Err(e) = update_new_entry(&suuid, item).await {
                    log::warn!("{e:?}");
                }
            });
        }

        ui.global::<Store>()
            .get_rss_lists()
            .set_row_data(index, rss);

        return;
    }
}
