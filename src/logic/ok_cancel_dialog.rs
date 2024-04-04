use crate::slint_generatedAppWindow::{AppWindow, Logic};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_handle_ok_cancel_dialog(move |handle_type, handle_uuid| {
            let ui = ui_handle.unwrap();

            match handle_type.as_str() {
                "remove-all-entrys" => {
                    ui.global::<Logic>().invoke_remove_all_entrys(handle_uuid);
                }
                "remove-all-favorite-entrys" => {
                    ui.global::<Logic>().invoke_remove_all_favorite_entrys();
                }
                "remove-rss" => {
                    ui.global::<Logic>().invoke_remove_rss(handle_uuid);
                }
                "remove-all-cache" => {
                    ui.global::<Logic>().invoke_remove_all_cache();
                }
                "backup-to-remote" => {
                    let setting = ui.global::<Logic>().invoke_get_setting_backup_recover();
                    ui.global::<Logic>().invoke_backup_to_remote(setting);
                }
                "recover-from-remote" => {
                    let setting = ui.global::<Logic>().invoke_get_setting_backup_recover();
                    ui.global::<Logic>().invoke_recover_from_remote(setting);
                }
                _ => (),
            }
        });
}
