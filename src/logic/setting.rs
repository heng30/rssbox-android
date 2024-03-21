use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::util::translator::tr;
use crate::{config, util};
use crate::{message_success, message_warn};
use slint::{ComponentHandle, Weak};

pub fn init(ui: &AppWindow) {
    init_setting_dialog(ui.as_weak());

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_clean_cache(move || {
        let ui = ui_handle.unwrap();
        match util::fs::remove_dir_files(&config::cache_dir()) {
            Err(e) => {
                message_warn!(ui, format!("{}. {}: {}", tr("清空失败"), tr("原因"), e));
            }
            _ => {
                message_success!(ui, tr("清空成功"));
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_setting_cancel(move || {
        init_setting_dialog(ui_handle.clone());
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_setting_ok(move |setting_config| {
        let ui = ui_handle.unwrap();
        let mut config = config::config();

        config.ui.font_size = setting_config
            .ui
            .font_size
            .to_string()
            .parse()
            .unwrap_or(18);
        config.ui.font_family = setting_config.ui.font_family.to_string();
        config.ui.win_width = u32::max(
            setting_config
                .ui
                .win_width
                .to_string()
                .parse()
                .unwrap_or(800),
            800,
        );
        config.ui.win_height = u32::max(
            setting_config
                .ui
                .win_height
                .to_string()
                .parse()
                .unwrap_or(600),
            600,
        );
        config.ui.language = setting_config.ui.language.to_string();

        config.account.max_feerate = setting_config
            .account
            .max_feerate
            .to_string()
            .parse()
            .unwrap_or(100);
        config.account.max_fee_amount = setting_config
            .account
            .max_fee_amount
            .to_string()
            .parse()
            .unwrap_or(10_000);
        config.account.max_send_amount = setting_config
            .account
            .max_send_amount
            .to_string()
            .parse()
            .unwrap_or(1_f64);
        config.account.skip_utxo_amount = setting_config
            .account
            .skip_utxo_amount
            .to_string()
            .parse()
            .unwrap_or(1_000);

        config.socks5.enabled = setting_config.proxy.enabled;
        config.socks5.url = setting_config.proxy.url.to_string();
        config.socks5.port = setting_config
            .proxy
            .port
            .to_string()
            .parse()
            .unwrap_or(1080);

        match config::save(config) {
            Err(e) => {
                message_warn!(ui, format!("{}, {}: {:?}", tr("保存失败"), tr("原因"), e));
            }
            _ => {
                init_setting_dialog(ui.as_weak());
                message_success!(ui, tr("保存成功"));
            }
        }
    });
}

fn init_setting_dialog(ui: Weak<AppWindow>) {
    let ui = ui.unwrap();
    let ui_config = config::ui();
    let socks5_config = config::socks5();
    let account_config = config::account();

    let mut setting_dialog = ui.global::<Store>().get_setting_dialog_config();
    setting_dialog.ui.font_size = slint::format!("{}", ui_config.font_size);
    setting_dialog.ui.font_family = ui_config.font_family.into();
    setting_dialog.ui.win_width = slint::format!("{}", u32::max(ui_config.win_width, 800));
    setting_dialog.ui.win_height = slint::format!("{}", u32::max(ui_config.win_height, 600));
    setting_dialog.ui.language = ui_config.language.into();

    setting_dialog.account.max_feerate = slint::format!("{}", account_config.max_feerate);
    setting_dialog.account.max_fee_amount = slint::format!("{}", account_config.max_fee_amount);
    setting_dialog.account.max_send_amount = slint::format!("{}", account_config.max_send_amount);
    setting_dialog.account.skip_utxo_amount = slint::format!("{}", account_config.skip_utxo_amount);

    setting_dialog.proxy.enabled = socks5_config.enabled;
    setting_dialog.proxy.url = socks5_config.url.into();
    setting_dialog.proxy.port = slint::format!("{}", socks5_config.port);

    ui.global::<Store>()
        .set_setting_dialog_config(setting_dialog);
}
