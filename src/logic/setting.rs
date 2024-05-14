use super::{
    entry,
    message::{async_message_success, async_message_warn},
    rss, ReqData,
};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, SettingBackupRecover, SettingProxy, SettingReading, SettingSync,
    SettingUpdate, Store, Theme,
};
use crate::{
    config::{self, Config},
    db::{self, entry::RssEntry, rss::RssConfig},
    message_warn,
    util::{http, translator::tr},
    version,
};
use anyhow::Result;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Weak};
use std::time::Duration;
use uuid::Uuid;

const FEEDBACK_URL: &str = "https://heng30.xyz/apisvr/rssbox/android/feedback";
const BACKUP_URL: &str = "https://heng30.xyz/apisvr/rssbox/android/backup";
const RECOVER_URL: &str = "https://heng30.xyz/apisvr/rssbox/android/recover";
const LATEST_VERSION_URL: &str = "https://heng30.xyz/apisvr/latest/version?q=rssbox-android";

// const BACKUP_URL: &str = "http://127.0.0.1:8004/rssbox/android/backup";
// const RECOVER_URL: &str = "http://127.0.0.1:8004/rssbox/android/recover";

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct BackupRecoverData {
    rss: Vec<RssConfig>,
    collection: Vec<RssEntry>,
    setting: Config,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct SettingUpdateData {
    #[serde(skip)]
    current_version: String,
    latest_version: String,
    detail_cn: String,
    detail_en: String,
    url: String,
}

impl From<SettingUpdateData> for SettingUpdate {
    fn from(setting: SettingUpdateData) -> Self {
        Self {
            current_version: setting.current_version.into(),
            latest_version: setting.latest_version.into(),
            detail_cn: setting.detail_cn.into(),
            detail_en: setting.detail_en.into(),
            url: setting.url.into(),
        }
    }
}

pub fn init(ui: &AppWindow) {
    init_setting(&ui);

    ui.global::<Store>()
        .set_is_first_run(config::is_first_run());

    ui.global::<Store>().set_setting_update(SettingUpdate {
        current_version: version::VERSION.into(),
        latest_version: version::VERSION.into(),
        ..Default::default()
    });

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
        all.ui.is_dark = setting.is_dark;
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
        let (ui, text) = (ui_handle.unwrap(), text.trim().to_string());
        if text.is_empty() {
            message_warn!(ui, tr("非法输入"));
            return;
        }

        let ui = ui.as_weak();
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

    ui.global::<Logic>().on_get_setting_backup_recover(move || {
        let config = config::backup_recover();

        SettingBackupRecover {
            api_token: config.api_token.into(),
            favorite: config.favorite,
            rss: config.rss,
            setting: config.setting,
        }
    });

    ui.global::<Logic>()
        .on_set_setting_backup_recover(move |setting| {
            let mut all = config::all();

            all.backup_recover.api_token = setting.api_token.into();
            all.backup_recover.favorite = setting.favorite;
            all.backup_recover.rss = setting.rss;
            all.backup_recover.setting = setting.setting;

            _ = config::save(all);
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_backup_to_remote(move |options| {
        let ui = ui_handle.unwrap();
        let mut data = BackupRecoverData::default();

        if options.rss {
            data.rss = rss::get_rss_configs(&ui);
        }

        if options.favorite {
            data.collection = entry::get_favorite_entrys(&ui);
        }

        if options.setting {
            data.setting = config::all();
        }

        backup_to_remote(ui.as_weak(), options.api_token.into(), data);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_recover_from_remote(move |options| {
        recover_from_remote(ui_handle.clone(), options);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_get_setting_update(move || {
        _get_setting_update(ui_handle.clone());
    });
}

fn init_setting(ui: &AppWindow) {
    let config = config::ui();
    let mut ui_setting = ui.global::<Store>().get_setting_ui();

    let font_size = u32::min(50, u32::max(10, config.font_size));
    ui_setting.font_size = slint::format!("{}", font_size);
    ui_setting.font_family = config.font_family.into();
    ui_setting.language = config.language.into();
    ui_setting.is_dark = config.is_dark;

    ui.global::<Theme>().invoke_set_dark(config.is_dark);
    ui.global::<Store>().set_setting_ui(ui_setting);
}

async fn _send_feedback(text: String) -> Result<()> {
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

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    let res = http::client(None)?
        .post(FEEDBACK_URL)
        .timeout(Duration::from_secs(15))
        .headers(headers)
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

fn backup_to_remote(ui: Weak<AppWindow>, api_token: String, data: BackupRecoverData) {
    tokio::spawn(async move {
        match _send_backup_to_remove(api_token, data).await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("备份失败"), tr("原因")),
            ),
            _ => async_message_success(ui.clone(), tr("备份成功")),
        }
    });
}

async fn _send_backup_to_remove(api_token: String, data: BackupRecoverData) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {api_token}").parse().unwrap(),
    );

    let url = format!("{BACKUP_URL}?api_token={api_token}");
    let res = http::client(None)?
        .post(&url)
        .timeout(Duration::from_secs(15))
        .headers(headers)
        .json(&data)
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

async fn _fetch_backup_from_remove(api_token: String) -> Result<BackupRecoverData> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {api_token}").parse().unwrap(),
    );

    let url = format!("{RECOVER_URL}?api_token={api_token}");
    Ok(http::client(None)?
        .get(&url)
        .timeout(Duration::from_secs(15))
        .headers(headers)
        .send()
        .await?
        .json::<BackupRecoverData>()
        .await?)
}

fn recover_from_remote(ui: Weak<AppWindow>, options: SettingBackupRecover) {
    tokio::spawn(async move {
        match _fetch_backup_from_remove(options.api_token.into()).await {
            Ok(data) => {
                if options.setting {
                    config::reset(data.setting);

                    let ui = ui.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        init_setting(&ui.unwrap());
                    });
                }

                let rss = data.rss;
                if options.rss {
                    _ = db::rss::delete_all().await;

                    let ui = ui.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        let ui = ui.unwrap();
                        rss::remove_all_rss(&ui);

                        for item in rss.into_iter() {
                            ui.global::<Logic>().invoke_new_rss(item.into());
                        }
                    });
                }

                if options.favorite {
                    _ = db::entry::delete_all(entry::FAVORITE_UUID).await;
                    for item in data.collection.into_iter() {
                        let uuid = Uuid::new_v4().to_string();
                        if let Ok(text) = serde_json::to_string(&item) {
                            _ = db::entry::insert(entry::FAVORITE_UUID, &uuid, &text).await;
                        };
                    }

                    let ui = ui.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        entry::init_favorite(ui.clone());
                    });
                }

                async_message_success(ui.clone(), tr("恢复成功"));
            }
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("恢复失败"), tr("原因")),
            ),
        }
    });
}

async fn _inner_get_setting_update() -> Result<SettingUpdateData> {
    Ok(http::client(None)?
        .get(LATEST_VERSION_URL)
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .json::<SettingUpdateData>()
        .await?)
}

fn _get_setting_update(ui: Weak<AppWindow>) {
    tokio::spawn(async move {
        match _inner_get_setting_update().await {
            Err(e) => async_message_warn(
                ui.clone(),
                format!("{}. {}: {e:?}", tr("获取最新版本信息失败"), tr("原因")),
            ),
            Ok(mut v) => {
                v.current_version = version::VERSION.into();

                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    ui.unwrap().global::<Store>().set_setting_update(v.into());
                });
            }
        }
    });
}
