extern crate rssbox;

use anyhow::Result;
use atom_syndication::Feed;
use rss::Channel;
use rssbox::{logic::top_rss_list_cn, util::http};
use std::{collections::HashSet, io::BufReader, time::Duration};
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc};

const TOP_RSS_LIST_CN: &str = include_str!("../data/top-rss-list.json");
const TOP_RSS_LIST_CN_VALID: &str = "./data/top-rss-list-valid.json";

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() -> Result<()> {
    log::info!("start...");
    rssbox::init_logger();

    let _assert = File::create(TOP_RSS_LIST_CN_VALID).await?;

    let items = top_rss_list_cn(TOP_RSS_LIST_CN)?;
    let total_len = items.len();
    let (tx, mut rx) = mpsc::channel(total_len);

    for (index, item) in items.into_iter().enumerate() {
        let tx = tx.clone();
        tokio::spawn(async move {
            match fetch_rss(&item.url).await {
                Ok(content) => {
                    if let Ok(channel) = Channel::read_from(&content[..]) {
                        if !channel.items.is_empty() {
                            _ = tx.send(item).await;
                        }
                    } else if let Ok(feed) = Feed::read_from(BufReader::new(&content[..])) {
                        if !feed.entries.is_empty() {
                            _ = tx.send(item).await;
                        }
                    }
                }
                _ => (),
            }
            log::info!("{}/{total_len}", index + 1);
        });
    }

    drop(tx);

    let mut valid_items = HashSet::new();
    while let Some(item) = rx.recv().await {
        valid_items.insert(item);
    }
    let valid_items = valid_items.into_iter().collect::<Vec<_>>();

    let text = serde_json::to_string::<Vec<_>>(&valid_items)?;
    log::info!("{text}");
    log::info!("total items: {}", valid_items.len());

    let mut file = File::create(TOP_RSS_LIST_CN_VALID).await?;
    file.write_all(text.as_bytes()).await?;

    log::info!("exit...");

    Ok(())
}

async fn fetch_rss(url: &str) -> Result<Vec<u8>> {
    Ok(http::client(None)?
        .get(url)
        .headers(http::headers())
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .bytes()
        .await?
        .to_vec())
}
