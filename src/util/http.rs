use crate::config;
use reqwest::{
    header::{HeaderMap, ACCEPT, CACHE_CONTROL, USER_AGENT},
    Client, Proxy, Result,
};

pub enum ProxyType {
    Http,
    Socks5,
    Unknown,
}

impl From<&str> for ProxyType {
    fn from(pt: &str) -> Self {
        match pt.to_lowercase().as_str() {
            "http" => ProxyType::Http,
            "socks5" => ProxyType::Socks5,
            _ => ProxyType::Unknown,
        }
    }
}

pub fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert(ACCEPT, "*/*".parse().unwrap());

    headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());
    headers
}

pub fn client(proxy_type: Option<ProxyType>) -> Result<Client> {
    match proxy_type {
        None => Ok(Client::new()),
        Some(item) => {
            let config = config::proxy();

            let proxy = match item {
                ProxyType::Http => {
                    Proxy::all(format!("http://{}:{}", config.http_url, config.http_port))?
                }
                ProxyType::Socks5 => Proxy::all(format!(
                    "socks5://{}:{}",
                    config.socks5_url, config.socks5_port
                ))?,
                _ => return Ok(Client::new()),
            };
            Ok(Client::builder().proxy(proxy).build()?)
        }
    }
}
