// use super::data::SyncItem;
use super::message::{async_message_success, async_message_warn};
use crate::slint_generatedAppWindow::{
    AppWindow, Logic, RssConfig as UIRssConfig, RssEntry as UIRssEntry, Store,
};
use crate::{
    config,
    db::{self, entry::RssEntry, rss::RssConfig, ComEntry},
    message_success, message_warn,
    util::{self, crypto::md5_hex, http, translator::tr},
};
use anyhow::Result;
use atom_syndication::{Feed, FixedDateTime, Link, TextType};
use html2text;
use rss::Channel;
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};
use std::{cmp::Ordering, io::BufReader, time::Duration};
use uuid::Uuid;

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

fn init_rss(ui: &AppWindow) {
    ui.global::<Store>().set_rss_lists(ModelRc::default());
    ui.global::<Store>().set_rss_entrys(ModelRc::default());

    let ui = ui.as_weak();
    tokio::spawn(async move {
        match db::rss::select_all().await {
            Ok(items) => {
                let config_list = init_rss_configs(items).await;
                let entry_list = init_rss_entrys(&config_list).await;
                assert_eq!(config_list.len(), entry_list.len());

                let ui = ui.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let ui = ui.unwrap();
                    let mut list = vec![];

                    for (index, item) in config_list.into_iter().enumerate() {
                        let mut item: UIRssConfig = item.into();
                        item.unread_counts = rss_entrys_unread_counts(&entry_list[index]);
                        item.entry = ModelRc::new(VecModel::from(entry_list[index].clone()));

                        if index == 0 {
                            ui.global::<Store>().set_rss_entrys(item.entry.clone());
                            ui.global::<Store>().set_current_rss_uuid(item.uuid.clone());
                        }

                        list.push(item);
                    }

                    list.sort_by(rss_config_sort_fn);
                    ui.global::<Store>()
                        .set_rss_lists(ModelRc::new(VecModel::from(list)));
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

                ui.global::<Store>()
                    .get_rss_lists()
                    .as_any()
                    .downcast_ref::<VecModel<UIRssConfig>>()
                    .expect("We know we set a VecModel earlier")
                    .set_vec(list);

                message_success!(ui, ("新建成功"));
            });
        });
    });
}

async fn _new_rss(mut rss: RssConfig) -> Result<RssConfig> {
    rss.uuid = Uuid::new_v4().to_string().into();
    let config = serde_json::to_string(&rss)?;
    db::rss::insert(rss.uuid.as_str(), &config).await?;
    Ok(rss)
}
