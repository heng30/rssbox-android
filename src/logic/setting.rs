use crate::slint_generatedAppWindow::{
    AppWindow, Logic, SettingProxy, SettingSync, SettingUI, Store,
};
use crate::{
    config, message_success, message_warn,
    util::{self, translator::tr},
};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    init_setting(&ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_get_setting_ui(move || {
        let ui = ui_handle.unwrap();
        ui.global::<Store>().get_setting_ui()
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_set_setting_ui(move |mut setting| {
        let font_size = u32::min(50, u32::max(10, setting.font_size.parse().unwrap_or(16)));
        setting.font_size = slint::format!("{}", font_size);

        ui_handle
            .unwrap()
            .global::<Store>()
            .set_setting_ui(setting.clone());

        let mut all = config::all();
        all.ui.font_size = font_size.into();
        all.ui.font_family = setting.font_family.into();
        all.ui.language = setting.language.into();
        _ = config::save(all);
    });

    ui.global::<Logic>().on_get_setting_sync(move || {
        let config = config::sync();
        SettingSync {
            sync_interval: slint::format!("{}", config.sync_interval),
            sync_timeout: slint::format!("{}", config.sync_timeout),
            is_auto_sync: config.is_auto_sync,
            is_start_sync: config.is_start_sync,
        }
    });

    ui.global::<Logic>().on_set_setting_sync(move |setting| {
        let mut all = config::all();

        all.sync.sync_interval = setting.sync_interval.parse().unwrap_or(60);
        all.sync.sync_timeout = setting.sync_timeout.parse().unwrap_or(15);
        all.sync.is_auto_sync = setting.is_auto_sync;
        all.sync.is_start_sync = setting.is_start_sync;
        _ = config::save(all);
    });

    ui.global::<Logic>().on_get_setting_proxy(move || {
        let config = config::proxy();

        SettingProxy {
            proxy_type: "Http".into(),
            http_url: config.http_url.into(),
            http_port: slint::format!("{}", config.http_port),
            socks5_url: config.socks5_url.into(),
            socks5_port: slint::format!("{}", config.socks5_port),
        }
    });

    ui.global::<Logic>().on_set_setting_proxy(move |setting| {
        let mut all = config::all();

        all.proxy.http_url = setting.http_url.into();
        all.proxy.http_port = setting.http_port.parse().unwrap_or(3218);
        all.proxy.socks5_url = setting.socks5_url.into();
        all.proxy.socks5_port = setting.socks5_port.parse().unwrap_or(1080);
        _ = config::save(all);
    });

    ui.global::<Logic>()
        .on_tr(move |_is_cn, text| tr(text.as_str()).into());
}

fn init_setting(ui: &AppWindow) {
    let config = config::ui();
    let mut ui_setting = ui.global::<Store>().get_setting_ui();
    ui_setting.font_size = slint::format!("{}", config.font_size);
    ui_setting.font_family = config.font_family.into();
    ui_setting.language = config.language.into();

    ui.global::<Store>().set_setting_ui(ui_setting);
}
