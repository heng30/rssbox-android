extern crate rssbox;

use anyhow::{Context, Result};
use atom_syndication::Feed;
use clap::Parser;
use opml::OPML;
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use rss::Channel;
use rssbox::{
    db::ComEntry,
    logic::{top_rss_list_cn, FindEntry},
    util::http,
};
use std::{collections::HashSet, fs, io::BufReader, time::Duration};
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc};

const TOP_RSS_LIST_CN_OPML: &str = include_str!("../data/rss-list.opml");
const TOP_RSS_LIST_CN: &str = include_str!("../data/top-rss-list.json");
const TOP_RSS_LIST_CN_VALID_PATH: &str = "./data/top-rss-list-valid.json";

/// Tool program to generate valid rss and send to api server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Generate valid rss
    #[arg(short, long, default_value_t = false)]
    generate: bool,

    /// API server root url
    #[arg(short, long, default_value_t = String::default())]
    root_url: String,
}

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() -> Result<()> {
    rssbox::init_logger();

    let args = Args::parse();
    if args.generate {
        generate_valid_rss()
            .await
            .context("generate valid rss failed")?;
    }

    if !args.root_url.is_empty() {
        update_apisvr_rss(&args.root_url)
            .await
            .context("update apisvr rss failed")?;
    }

    Ok(())
}

async fn update_apisvr_rss(root_url: &str) -> Result<()> {
    log::info!("update start...");

    let get_url = format!("{root_url}/rssbox/rss/list/cn");

    log::info!("get url: {}", get_url);

    let remote_items = http::client(None)?
        .get(get_url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json::<Vec<ComEntry>>()
        .await?
        .into_iter()
        .filter_map(|item| serde_json::from_str::<FindEntry>(&item.data).ok())
        .collect::<HashSet<_>>();

    log::info!("remote_items: {}", remote_items.len());

    let json_text =
        fs::read_to_string(TOP_RSS_LIST_CN_VALID_PATH).context("read valid json file failed")?;

    let local_items = top_rss_list_cn(&json_text)
        .context("parse json file error")?
        .into_iter()
        .collect::<HashSet<_>>();

    log::info!("local_items: {}", local_items.len());

    let difference_items_rl: HashSet<_> = remote_items.difference(&local_items).cloned().collect();
    let difference_items_lr: HashSet<_> = local_items.difference(&remote_items).cloned().collect();

    let difference_items = difference_items_rl
        .into_iter()
        .chain(difference_items_lr.into_iter())
        .collect::<Vec<_>>();

    let difference_items_len = difference_items.len();
    log::info!("difference_items: {}", difference_items_len);

    if difference_items_len == 0 {
        return Ok(());
    }

    let (tx, mut rx) = mpsc::channel(difference_items_len);

    for item in difference_items.into_iter() {
        let (root_url, tx) = (root_url.to_string(), tx.clone());

        tokio::spawn(async move {
            let post_url = format!("{root_url}/rssbox/rss/list/cn");
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

            _ = http::client(None)
                .expect("create http client failed")
                .post(post_url)
                .timeout(Duration::from_secs(30))
                .headers(headers)
                .json(&item)
                .send()
                .await;

            _ = tx.send(()).await;
        });
    }

    drop(tx);

    let mut count = 1;
    while let Some(_) = rx.recv().await {
        log::info!("{count}/{difference_items_len}");
        count += 1;
    }

    log::info!("update exit...");

    Ok(())
}

async fn generate_valid_rss() -> Result<()> {
    log::info!("generate start...");

    let _assert = File::create(TOP_RSS_LIST_CN_VALID_PATH).await?;

    let json_items = top_rss_list_cn(TOP_RSS_LIST_CN).context("parse json file error")?;
    log::info!("{}", json_items.len());
    assert!(!json_items.is_empty());

    let opml_items = parse_opml(TOP_RSS_LIST_CN_OPML).context("parse opml file error")?;
    log::info!("{}", opml_items.len());
    assert!(!opml_items.is_empty());

    let items = json_items
        .into_iter()
        .chain(opml_items)
        .into_iter()
        .collect::<Vec<_>>();

    log::info!("{}", items.len());
    assert!(!items.is_empty());

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
    // log::info!("{text}");
    log::info!("total items: {}", valid_items.len());

    let mut file = File::create(TOP_RSS_LIST_CN_VALID_PATH).await?;
    file.write_all(text.as_bytes()).await?;

    log::info!("generate exit...");

    Ok(())
}

async fn fetch_rss(url: &str) -> Result<Vec<u8>> {
    Ok(http::client(None)?
        .get(url)
        .headers(http::headers())
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .bytes()
        .await?
        .to_vec())
}

fn parse_opml(text: &str) -> Result<Vec<FindEntry>> {
    Ok(OPML::from_str(text)?
        .body
        .outlines
        .into_iter()
        .filter(|item| !item.text.is_empty() && item.xml_url.is_some())
        .map(|item| FindEntry {
            name: item.text,
            url: item.xml_url.unwrap(),
        })
        .collect())
}
