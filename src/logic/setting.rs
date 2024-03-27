use super::{
    message::{async_message_success, async_message_warn},
    ReqData,
};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, SettingProxy, SettingReading, SettingSync, Store,
};
use crate::{
    config,
    util::{http, translator::tr},
};
use anyhow::Result;
use slint::{ComponentHandle, SharedString};
use std::time::Duration;

const FEEDBACK_URL: &str = "https://heng30.xyz/apisvr/rssbox/android/feedback";

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

    ui.global::<Logic>().on_get_setting_reading(move || {
        let config = config::reading();

        SettingReading {
            browser: config.browser.into(),
            is_delete_after_reading: config.is_delete_after_reading,
        }
    });

    ui.global::<Logic>().on_set_setting_reading(move |setting| {
        let mut all = config::all();

        all.reading.browser = setting.browser.into();
        all.reading.is_delete_after_reading = setting.is_delete_after_reading;
        _ = config::save(all);
    });

    ui.global::<Logic>()
        .on_tr(move |_is_cn, text| tr(text.as_str()).into());

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_send_feedback(move |text| {
        let ui = ui_handle.clone();
        tokio::spawn(async move {
            match _send_feedback(text).await {
                Err(e) => async_message_warn(
                    ui.clone(),
                    format!("{}. {}: {e:?}", tr("发送失败"), tr("原因")),
                ),
                _ => async_message_success(ui.clone(), tr("发送成功")),
            }
        });
    });
}

fn init_setting(ui: &AppWindow) {
    let config = config::ui();
    let mut ui_setting = ui.global::<Store>().get_setting_ui();

    let font_size = u32::min(50, u32::max(10, config.font_size));
    ui_setting.font_size = slint::format!("{}", font_size);
    ui_setting.font_family = config.font_family.into();
    ui_setting.language = config.language.into();

    ui.global::<Store>().set_setting_ui(ui_setting);
}

async fn _send_feedback(text: SharedString) -> Result<()> {
    let chars_text = text.chars().collect::<Vec<_>>();
    let text = if chars_text.len() > 2048 {
        format!("{}", chars_text[..2048].iter().collect::<String>())
    } else {
        text.into()
    };

    let req = ReqData {
        r#type: "feedback".into(),
        data: text,
        ..Default::default()
    };

    let res = http::client(None)?
        .post(FEEDBACK_URL)
        .timeout(Duration::from_secs(15))
        .json(&req)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!(
            "http error code: {}",
            res.status().as_str()
        ));
    }

    Ok(())
}
