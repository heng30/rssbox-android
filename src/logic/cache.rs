use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::{
    db,
    util::{self, translator::tr},
};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    init_cache(ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_all_cache(move || {
        ui_handle
            .unwrap()
            .global::<Store>()
            .set_cache_size("0M".into());

        let ui = ui_handle.clone();
        tokio::spawn(async move {
            match db::trash::delete_all().await {
                Err(e) => async_message_warn(
                    ui.clone(),
                    format!("{}. {}: {e:?}", tr("清除缓存失败"), tr("原因")),
                ),
                _ => async_message_success(ui.clone(), tr("清除缓存成功")),
            }
        });
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_update_cache_size(move || {
        init_cache(&ui_handle.unwrap());
    });
}

fn init_cache(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    tokio::spawn(async move {
        let ui = ui_handle.clone();
        match db::trash::row_count().await {
            Err(e) => log::warn!("Cache size error: {e:?}"),
            Ok(count) => {
                let _ = slint::invoke_from_event_loop(move || {
                    ui.clone()
                        .unwrap()
                        .global::<Store>()
                        .set_cache_size(util::str::pretty_size_string(count as u64 * 32).into());
                });
            }
        }
    });
}
