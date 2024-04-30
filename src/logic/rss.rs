use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, RssConfig as UIRssConfig, RssEntry as UIRssEntry, Store,
};
use crate::{
    config,
    db::{self, entry::RssEntry, rss::RssConfig, ComEntry},
    message_info, message_success, store_rss_entrys,
    util::{self, crypto::md5_hex, http, translator::tr},
};
use anyhow::{Context, Result};
use atom_syndication::{Feed, FixedDateTime, Link, TextType};
use html2text;
use rss::Channel;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};
use std::{cmp::Ordering, io::BufReader, time::Duration};
use uuid::Uuid;

const EMPTY_UUID: &str = "empty-uuid";

#[derive(Debug, Default, Clone)]
pub struct SyncItem {
    pub suuid: String,
    pub url: String,
    pub proxy_type: String,
    pub feed_format: String,
}

#[derive(Debug, Clone)]
struct ErrorMsg {
    url: String,
    msg: String,
}

impl From<UIRssConfig> for SyncItem {
    fn from(rss: UIRssConfig) -> Self {
        SyncItem {
            suuid: rss.uuid.to_string(),
            url: rss.url.to_string(),
            feed_format: rss.feed_format.to_string(),
            proxy_type: if rss.use_http_proxy {
                "Http".to_string()
            } else if rss.use_socks5_proxy {
                "Socks5".to_string()
            } else {
                "Unknown".to_string()
            },
        }
    }
}

#[macro_export]
macro_rules! store_rss_lists {
    ($ui:expr) => {
        $ui.global::<Store>()
            .get_rss_lists()
            .as_any()
            .downcast_ref::<VecModel<UIRssConfig>>()
            .expect("We know we set a VecModel earlier")
    };
}

pub fn get_rss_config(ui: &AppWindow, uuid: &str) -> Option<UIRssConfig> {
    for rss in ui.global::<Store>().get_rss_lists().iter() {
        if rss.uuid != uuid {
            continue;
        }

        return Some(rss);
    }

    None
}

pub fn get_rss_configs(ui: &AppWindow) -> Vec<RssConfig> {
    ui.global::<Store>()
        .get_rss_lists()
        .iter()
        .map(|item| item.clone().into())
        .collect()
}

pub fn is_exist_url(ui: &AppWindow, url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    ui.global::<Store>()
        .get_rss_lists()
        .iter()
        .find(|item| item.url == url)
        .is_some()
}

pub fn decease_unread_counts(ui: &AppWindow, uuid: &str) {
    for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
        if rss.uuid != uuid {
            continue;
        }

        rss.unread_counts = i32::max(0, rss.unread_counts - 1);
        ui.global::<Store>()
            .get_rss_lists()
            .set_row_data(index, rss);

        notify_ui_update_unread_counts(&ui);
        return;
    }
}

pub fn reset_unread_counts(ui: &AppWindow, uuid: &str) {
    for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
        if rss.uuid != uuid {
            continue;
        }

        rss.unread_counts = 0;
        ui.global::<Store>()
            .get_rss_lists()
            .set_row_data(index, rss);

        notify_ui_update_unread_counts(ui);
        return;
    }
}

pub fn notify_ui_update_unread_counts(ui: &AppWindow) {
    ui.global::<Store>()
        .set_rss_unread_counts_flag(!ui.global::<Store>().get_rss_unread_counts_flag());
}

async fn init_rss_configs(items: Vec<ComEntry>) -> Vec<RssConfig> {
    let mut list = vec![];
    for item in items.into_iter() {
        let rss_config = match serde_json::from_str::<RssConfig>(&item.data) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("{:?}", e);
                continue;
            }
        };

        list.push(rss_config);
    }

    list
}

async fn init_rss_entrys(items: &Vec<RssConfig>) -> Vec<Vec<UIRssEntry>> {
    let mut list = vec![];
    for item in items.iter() {
        list.push(super::entry::get_from_db(item.uuid.as_str()).await);
    }

    list
}

fn rss_entrys_unread_counts(items: &Vec<UIRssEntry>) -> i32 {
    let mut unread_counts = 0;
    for item in items.iter() {
        if !item.is_read {
            unread_counts += 1;
        }
    }

    unread_counts
}

fn rss_config_sort_fn(a: &UIRssConfig, b: &UIRssConfig) -> Ordering {
    if a.is_favorite && b.is_favorite {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    } else if a.is_favorite && !b.is_favorite {
        Ordering::Less
    } else if !a.is_favorite && b.is_favorite {
        Ordering::Greater
    } else {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    }
}

fn rss_config_sort(ui: &AppWindow) {
    let mut list = ui
        .global::<Store>()
        .get_rss_lists()
        .iter()
        .collect::<Vec<_>>();

    list.sort_by(rss_config_sort_fn);
    store_rss_lists!(ui).set_vec(list);
}

pub fn remove_all_rss(ui: &AppWindow) {
    let uuids = ui
        .global::<Store>()
        .get_rss_lists()
        .iter()
        .map(|item| item.uuid)
        .collect::<Vec<_>>();

    for uuid in uuids.into_iter() {
        ui.global::<Logic>().invoke_remove_rss(uuid);
    }
}

fn update_rss_config_from_ui(src_config: &mut UIRssConfig, ui_config: UIRssConfig) {
    src_config.name = ui_config.name;
    src_config.url = ui_config.url;
    src_config.use_http_proxy = ui_config.use_http_proxy;
    src_config.use_socks5_proxy = ui_config.use_socks5_proxy;
    src_config.icon_index = ui_config.icon_index;
    src_config.feed_format = ui_config.feed_format;
    src_config.is_favorite = ui_config.is_favorite;
}

pub fn init_rss(ui: &AppWindow) {
    store_rss_lists!(ui).set_vec(vec![]);
    store_rss_entrys!(ui).set_vec(vec![]);

    let ui_handle = ui.as_weak();
    tokio::spawn(async move {
        match db::rss::select_all().await {
            Ok(items) => {
                let config_list = init_rss_configs(items).await;
                let entry_list = init_rss_entrys(&config_list).await;
                assert_eq!(config_list.len(), entry_list.len());

                let ui = ui_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    let mut list = vec![];

                    for (index, item) in config_list.into_iter().enumerate() {
                        let mut item: UIRssConfig = item.into();
                        item.unread_counts = rss_entrys_unread_counts(&entry_list[index]);
                        item.entry = ModelRc::new(VecModel::from(entry_list[index].clone()));

                        list.push(item);
                    }

                    list.sort_by(rss_config_sort_fn);

                    if !list.is_empty() {
                        ui.global::<Store>().set_rss_entrys(list[0].entry.clone());
                        ui.global::<Store>()
                            .set_current_rss_uuid(list[0].uuid.clone());
                    }

                    store_rss_lists!(ui).set_vec(list);

                    if config::sync().is_start_sync {
                        ui.global::<Logic>().invoke_sync_rss_all();
                    }
                });
            }
            Err(e) => log::warn!("{e:?}"),
        }
    });
}

pub fn init(ui: &AppWindow) {
    init_rss(ui);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_new_rss(move |config| {
        let ui = ui_handle.unwrap();

        if is_exist_url(&ui, &config.url) {
            message_info!(ui, tr("请勿重复添加"));
            return;
        }

        let rss: RssConfig = config.into();

        let ui = ui.as_weak();
        tokio::spawn(async move {
            let rss = match _new_rss(rss).await {
                Err(e) => {
                    async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("新建失败"), tr("原因")),
                    );
                    return;
                }
                Ok(item) => item,
            };

            let _ = slint::invoke_from_event_loop(move || {
                let ui = ui.clone().unwrap();
                let rss: UIRssConfig = rss.into();
                let suuid = rss.uuid.clone();

                let _assert = rss
                    .entry
                    .as_any()
                    .downcast_ref::<VecModel<UIRssEntry>>()
                    .expect("We know we set a VecModel earlier");

                let mut list = ui
                    .global::<Store>()
                    .get_rss_lists()
                    .iter()
                    .collect::<Vec<_>>();

                if list.is_empty() {
                    ui.global::<Store>().set_rss_entrys(rss.entry.clone());
                    ui.global::<Store>().set_current_rss_uuid(rss.uuid.clone());
                }

                list.push(rss);
                list.sort_by(rss_config_sort_fn);

                store_rss_lists!(ui).set_vec(list);
                message_success!(ui, tr("新建成功"));

                ui.global::<Logic>().invoke_sync_rss(suuid, true);
            });
        });
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_update_rss(move |uuid, config| {
        let ui = ui_handle.unwrap();

        for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
            if rss.uuid != uuid {
                continue;
            }

            update_rss_config_from_ui(&mut rss, config);

            ui.global::<Store>()
                .get_rss_lists()
                .set_row_data(index, rss.clone());

            let ui = ui.as_weak();
            let rss = RssConfig::from(rss);
            tokio::spawn(async move {
                match _edit_rss(rss).await {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("编辑失败"), tr("原因")),
                    ),
                    _ => async_message_success(ui.clone(), tr("编辑成功")),
                }
            });

            break;
        }

        rss_config_sort(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_remove_rss(move |uuid| {
        let ui = ui_handle.unwrap();

        for (index, rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
            if rss.uuid != uuid {
                continue;
            }

            store_rss_lists!(ui).remove(index);

            if uuid == ui.global::<Store>().get_current_rss_uuid() {
                ui.global::<Logic>().invoke_remove_all_entrys(uuid.clone());
                notify_ui_update_unread_counts(&ui);
                ui.global::<Logic>()
                    .invoke_switch_rss(uuid.clone(), EMPTY_UUID.into());
            }

            ui.global::<Logic>().invoke_add_to_find_blacklist(rss.url);

            let ui = ui.as_weak();
            tokio::spawn(async move {
                match _remove_rss(uuid.as_str()).await {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("删除失败"), tr("原因")),
                    ),
                    _ => async_message_success(ui.clone(), tr("删除成功")),
                }
            });

            return;
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_switch_rss(move |_from_uuid, to_uuid| {
            let ui = ui_handle.unwrap();
            let rss_lists = ui.global::<Store>().get_rss_lists();

            if rss_lists.row_count() == 0 {
                ui.global::<Store>().set_current_rss_uuid(EMPTY_UUID.into());
                return;
            }

            for rss in rss_lists.iter() {
                if to_uuid == EMPTY_UUID {
                    ui.global::<Store>().set_rss_entrys(rss.entry);
                    ui.global::<Store>().set_current_rss_uuid(rss.uuid);
                    break;
                }

                if rss.uuid == to_uuid {
                    ui.global::<Store>().set_rss_entrys(rss.entry);
                    ui.global::<Store>().set_current_rss_uuid(to_uuid);
                    break;
                }
            }
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_get_rss_config_to_ui(move |uuid| {
        let ui = ui_handle.unwrap();

        let mut des_rss = UIRssConfig::default();
        for rss in ui.global::<Store>().get_rss_lists().iter() {
            if rss.uuid != uuid {
                continue;
            }

            update_rss_config_from_ui(&mut des_rss, rss);
            break;
        }
        des_rss
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_name_rss(move |uuid| {
        get_rss_config(&ui_handle.unwrap(), &uuid).map_or(Default::default(), |item| item.name)
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_unread_counts_rss(move |uuid, _flag| {
            get_rss_config(&ui_handle.unwrap(), &uuid)
                .map_or(Default::default(), |item| item.unread_counts)
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_toggle_rss_favorite(move |uuid| {
        let ui = ui_handle.unwrap();

        for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
            if uuid != rss.uuid {
                continue;
            }

            rss.is_favorite = !rss.is_favorite;
            ui.global::<Store>()
                .get_rss_lists()
                .set_row_data(index, rss.clone());

            let ui = ui.as_weak();
            let rss = RssConfig::from(rss);
            tokio::spawn(async move {
                let is_favorite = rss.is_favorite;
                match _toggle_rss_favorite(rss).await {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!(
                            "{}. {}: {e:?}",
                            if is_favorite {
                                tr("收藏失败")
                            } else {
                                tr("取消收藏失败")
                            },
                            tr("原因")
                        ),
                    ),
                    _ => {
                        async_message_success(
                            ui.clone(),
                            if is_favorite {
                                tr("收藏成功")
                            } else {
                                tr("取消收藏成功")
                            },
                        );
                    }
                }
            });

            break;
        }
        rss_config_sort(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_update_time_rss(move |uuid, _flag| {
        let mut time = SharedString::default();

        for rss in ui_handle.unwrap().global::<Store>().get_rss_lists().iter() {
            if uuid != rss.uuid {
                continue;
            }

            time = rss.update_time;
            break;
        }

        time
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_exist_rss(move |uuid| get_rss_config(&ui_handle.unwrap(), uuid.as_str()).is_some());

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_sync_rss(move |suuid, is_show_toast| {
            let ui = ui_handle.unwrap();

            for (index, mut rss) in ui.global::<Store>().get_rss_lists().iter().enumerate() {
                if suuid != rss.uuid {
                    continue;
                }

                rss.is_update_failed = true;
                rss.update_time = util::time::local_now("%H:%M:%S").into();
                ui.global::<Store>()
                    .get_rss_lists()
                    .set_row_data(index, rss.clone());

                ui.global::<Store>()
                    .set_rss_update_time_flag(!ui.global::<Store>().get_rss_update_time_flag());

                let items: Vec<SyncItem> = vec![rss.into()];
                message_info!(ui, tr("正在同步..."));

                let ui = ui.as_weak();
                tokio::spawn(async move {
                    let error_msgs = sync_rss(ui.clone(), items).await;

                    if is_show_toast {
                        if error_msgs.is_empty() {
                            async_message_success(ui.clone(), tr("同步成功"));
                        } else {
                            let err = format!(
                                "{}:[{}] {}. {}: {}",
                                tr("访问"),
                                error_msgs[0].url,
                                tr("失败"),
                                tr("原因"),
                                error_msgs[0].msg
                            );
                            async_message_warn(ui.clone(), err);
                        }
                    }
                });

                return;
            }

            message_info!(&ui, tr("请添加RSS源"));
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_sync_rss_all(move || {
        let ui = ui_handle.unwrap();
        message_info!(ui, tr("正在同步..."));

        for item in ui.global::<Store>().get_rss_lists().iter() {
            ui.global::<Logic>().invoke_sync_rss(item.uuid, false);
        }
    });
}

async fn _new_rss(mut rss: RssConfig) -> Result<RssConfig> {
    rss.uuid = Uuid::new_v4().to_string().into();
    let config = serde_json::to_string(&rss)?;
    db::rss::insert(rss.uuid.as_str(), &config).await?;
    db::entry::new(rss.uuid.as_str()).await?;
    Ok(rss)
}

async fn _edit_rss(rss: RssConfig) -> Result<()> {
    let config = serde_json::to_string(&rss)?;
    db::rss::update(rss.uuid.as_str(), &config).await?;
    Ok(())
}

async fn _remove_rss(uuid: &str) -> Result<()> {
    db::rss::delete(uuid).await?;
    db::entry::drop_table(uuid).await?;
    Ok(())
}

async fn _toggle_rss_favorite(rss: RssConfig) -> Result<()> {
    let config = serde_json::to_string(&rss)?;
    db::rss::update(&rss.uuid.as_str(), &config).await?;
    Ok(())
}

fn parse_summary(summary: &str, is_text: bool) -> String {
    let mut max_counts = 100;
    let summary = summary.trim();

    let mut chars_summary = match is_text {
        true => summary.chars().collect::<Vec<_>>(),
        _ => html2text::from_read(summary.as_bytes(), usize::MAX)
            .replace("\n", "")
            .chars()
            .collect::<Vec<_>>(),
    };

    if chars_summary.len() > max_counts {
        chars_summary = chars_summary[..max_counts]
            .into_iter()
            .map(|v| v.clone())
            .collect::<Vec<_>>();
    }

    let summary = chars_summary.iter().collect::<String>();

    // contain none ascii chars
    if summary.len() > chars_summary.len() {
        max_counts = 50;
    };

    if chars_summary.len() > max_counts {
        format!(
            "{}...",
            chars_summary[..max_counts].iter().collect::<String>()
        )
    } else {
        format!("{}...", summary.to_string())
    }
}

fn parse_rss(suuid: &str, content: Vec<u8>) -> Result<Vec<RssEntry>> {
    let mut entrys = vec![];
    let ch = Channel::read_from(&content[..]).context("failed to parse rss xml")?;

    for item in ch.items() {
        let url = item.link().unwrap_or_default().to_string();
        let title = item.title().unwrap_or_default().to_string();
        let author = item.author().unwrap_or_default().to_string();
        let pub_date = item.pub_date().unwrap_or_default().to_string();

        let summary = match item.description() {
            Some(s) => parse_summary(s, false),
            _ => String::default(),
        };

        let tags = item
            .categories()
            .iter()
            .map(|c| c.name().to_string())
            .collect::<Vec<_>>()
            .join(",")
            .to_string();

        if url.is_empty() || title.is_empty() {
            continue;
        }

        entrys.push(RssEntry {
            suuid: suuid.to_string(),
            uuid: Uuid::new_v4().to_string(),
            url,
            title,
            pub_date,
            author,
            summary,
            tags,
            ..Default::default()
        });
    }

    Ok(entrys)
}

fn parse_atom(suuid: &str, content: Vec<u8>) -> Result<Vec<RssEntry>> {
    let mut entrys = vec![];
    let feed = Feed::read_from(BufReader::new(&content[..])).context("failed to parse atom xml")?;

    for item in feed.entries() {
        let url = item
            .links()
            .first()
            .unwrap_or(&Link::default())
            .href()
            .to_string();
        let title = item.title().as_str().to_string();
        let pub_date = item
            .published()
            .unwrap_or(&FixedDateTime::default())
            .to_string();

        let author = item
            .authors()
            .iter()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>()
            .join("|")
            .to_string();

        let summary = match item.summary() {
            Some(s) => parse_summary(s.as_str(), s.r#type == TextType::Text),
            _ => String::default(),
        };

        let tags = item
            .categories()
            .iter()
            .map(|c| c.term().to_string())
            .collect::<Vec<_>>()
            .join(",")
            .to_string();

        if url.is_empty() || title.is_empty() {
            continue;
        }

        entrys.push(RssEntry {
            suuid: suuid.to_string(),
            uuid: Uuid::new_v4().to_string(),
            url,
            title,
            pub_date,
            author,
            summary,
            tags,
            ..Default::default()
        });
    }

    Ok(entrys)
}

async fn fetch_entrys(sync_item: SyncItem) -> Result<Vec<RssEntry>> {
    let request_timeout = u64::max(config::sync().sync_timeout as u64, 10_u64);

    let client = http::client(Some(sync_item.proxy_type.as_str().into()))?;
    let content = client
        .get(&sync_item.url)
        .headers(http::headers())
        .timeout(Duration::from_secs(request_timeout))
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();

    let entrys = match sync_item.feed_format.to_lowercase().as_str() {
        "rss" => parse_rss(sync_item.suuid.as_str(), content)?,
        "atom" => parse_atom(sync_item.suuid.as_str(), content)?,
        _ => match parse_rss(sync_item.suuid.as_str(), content.clone()) {
            Ok(v) => v,
            _ => parse_atom(sync_item.suuid.as_str(), content)?,
        },
    };

    let mut unique_entrys = vec![];
    for item in entrys.into_iter() {
        if db::trash::is_exist(&md5_hex(item.url.as_str()))
            .await
            .is_err()
        {
            unique_entrys.push(item);
        }
    }

    let unique_entrys = unique_entrys.into_iter().rev().collect();
    Ok(unique_entrys)
}

async fn sync_rss(ui: Weak<AppWindow>, items: Vec<SyncItem>) -> Vec<ErrorMsg> {
    let mut error_msgs = vec![];

    for item in items.into_iter() {
        let (suuid, url) = (item.suuid.clone(), item.url.clone());

        match fetch_entrys(item).await {
            Ok(entrys) => {
                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    super::entry::update_new_entrys(&ui, suuid.as_str(), entrys);
                    if suuid.as_str() == ui.global::<Store>().get_current_rss_uuid().as_str() {
                        notify_ui_update_unread_counts(&ui);
                    }
                });
            }
            Err(e) => error_msgs.push(ErrorMsg {
                url,
                msg: format!("{e:?}"),
            }),
        }
    }

    error_msgs
}
