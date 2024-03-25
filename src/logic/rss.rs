use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, RssConfig as UIRssConfig, RssEntry as UIRssEntry, Store,
};
use crate::{
    config,
    db::{self, entry::RssEntry, rss::RssConfig, ComEntry},
    message_info, message_success, message_warn, store_rss_entrys,
    util::{self, crypto::md5_hex, http, translator::tr},
};
use anyhow::Result;
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
            } else {
                "Socks5".to_string()
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

fn update_rss_config_from_ui(src_config: &mut UIRssConfig, ui_config: UIRssConfig) {
    src_config.name = ui_config.name;
    src_config.url = ui_config.url;
    src_config.use_http_proxy = ui_config.use_http_proxy;
    src_config.use_socks5_proxy = ui_config.use_socks5_proxy;
    src_config.icon_index = ui_config.icon_index;
    src_config.feed_format = ui_config.feed_format;
    src_config.is_favorite = ui_config.is_favorite;
}

fn init_rss(ui: &AppWindow) {
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
        let rss: RssConfig = config.into();
        let ui = ui_handle.clone();

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

                let mut list = ui
                    .global::<Store>()
                    .get_rss_lists()
                    .iter()
                    .collect::<Vec<_>>();

                if list.is_empty() {
                    ui.global::<Store>().set_current_rss_uuid(rss.uuid.clone());
                }

                list.push(rss);
                list.sort_by(rss_config_sort_fn);

                store_rss_lists!(ui).set_vec(list);
                message_success!(ui, ("新建成功"));
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
                        format!("{}. {}: {e:?}", tr("新建失败"), tr("原因")),
                    ),
                    _ => async_message_success(ui.clone(), tr("新建成功")),
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

            ui.global::<Logic>().invoke_remove_all_entrys(uuid.clone());
            store_rss_lists!(ui).remove(index);

            if uuid == ui.global::<Store>().get_current_rss_uuid() {
                ui.global::<Logic>()
                    .invoke_switch_rss(uuid.clone(), EMPTY_UUID.into());
            }

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
            for rss in ui.global::<Store>().get_rss_lists().iter() {
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
                match _toggle_rss_favorite(rss).await {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("收藏失败"), tr("原因")),
                    ),
                    _ => async_message_success(ui.clone(), tr("收藏成功")),
                }
            });

            break;
        }
        rss_config_sort(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_update_time_rss(move |uuid| {
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
    ui.global::<Logic>().on_sync_rss(move |suuid| {
        let ui = ui_handle.unwrap();

        for rss in ui.global::<Store>().get_rss_lists().iter() {
            if suuid != rss.uuid {
                continue;
            }

            let mut items: Vec<SyncItem> = vec![rss.into()];
            message_info!(ui, tr("正在同步..."));

            let ui = ui.as_weak();
            tokio::spawn(async move {
                let error_msgs = sync_rss(ui, items).await;

                if error_msgs.is_empty() {
                    async_message_success(ui.clone(), tr("同步成功"));
                } else {
                    let err = format!("{}:[{}] {}. {}: {}"
                        tr("访问"), error_msgs[0].url, tr("失败"),
                        tr("原因"), error_msgs[0].msg);
                    async_message_warn(ui.clone(), err);
                }
            });

            return;
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

fn parse_rss(suuid: &str, content: Vec<u8>) -> Result<Vec<RssEntry>> {
    let mut entry = vec![];
    let ch = Channel::read_from(&content[..])?;

    for item in ch.items() {
        let url = item.link().unwrap_or_default().to_string();
        let title = item.title().unwrap_or_default().to_string();
        let author = item.author().unwrap_or_default().to_string();
        let pub_date = item.pub_date().unwrap_or_default().to_string();

        let summary = if item.description().is_some() {
            let s = item.description().unwrap();
            let s = html2text::from_read(s.as_bytes(), usize::MAX)
                .trim()
                .to_string();
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s
            }
        } else {
            String::default()
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

        entry.push(RssEntry {
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
}

fn parse_rss(suuid: &str, content: Vec<u8>) -> Result<Vec<RssEntry>> {
    let feed = Feed::read_from(BufReader::new(&content[..]))?;
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

        let summary = if item.summary().is_some() {
            let s = item.summary().unwrap();
            if s.r#type == TextType::Text {
                s.as_str().to_string()
            } else {
                String::default()
            }
        } else {
            String::default()
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

        entry.push(RssEntry {
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
}

async fn fetch_entry(config: SyncItem) -> Result<Vec<RssEntry>> {
    let rss_config = config::rss();
    let request_timeout = u64::min(rss_config.sync_timeout as u64, 10_u64);

    let client = uhttp::client(config.proxy_type.as_str().into())?;
    let content = client
        .get(&config.url)
        .headers(uhttp::headers())
        .timeout(Duration::from_secs(request_timeout))
        .send()
        .await?
        .bytes()
        .await?;

    let feed_format = config.feed_format.to_lowercase();

    let entrys = match feed_format {
        "auto" => match parse_rss(config.suuid.as_str(), content) {
            Ok(v) => v,
            _ => parse_atom(config.suuid.as_str(), content)?,
        },
        "rss" => parse_rss(config.suuid.as_str(), content)?,
        _ => parse_atom(config.suuid.as_str(), content)?,
    };

    entrys = entrys
        .into_iter()
        .filter(|e| match db::trash::is_exist(&md5_hex(e.url.as_str())) {
            Ok(flag) => !flag,
            _ => true,
        })
        .rev()
        .collect();

    Ok(entrys)
}

pub async fn sync_rss(ui: Weak<AppWindow>, items: Vec<SyncItem>) -> Vec<ErrorMsg> {
    let mut error_msgs = vec![];

    for item in items.into_iter() {
        let suuid = item.suuid.clone();
        let url = item.url.clone();

        match fetch_entry(item).await {
            Ok(entry) => {
                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    super::entry::update_new_entrys(&ui.unwrap(), suuid.as_str(), entry);
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
