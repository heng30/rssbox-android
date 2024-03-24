use crate::config;
use reqwest::{Client, Proxy, Result};

pub enum ProxyType {
    Http,
    Socks5,
    Unknow,
}

impl From<&str> for ProxyType {
    fn from(pt: &str) -> Self {
        match pt.to_lowercase().as_str() {
            "http" => ProxyType::Http,
            "socks5" => ProxyType::Socks5,
            _ => ProxyType::Unknow,
        }
    }
}

pub fn client(proxy_type: Option<ProxyType>) -> Result<Client> {
    match proxy_type {
        None => Ok(Client::new()),
        Some(item) => {
            let config = config::proxy();

            let proxy = match item {
                ProxyType::Http => {
                    Proxy::all(format!("https://{}:{}", config.http_url, config.http_port))?
                }
                ProxyType::Socks5 => Proxy::all(format!(
                    "socks5://{}:{}",
                    config.socks5_url, config.http_port
                ))?,
                _ => return Ok(Client::new()),
            };
            Ok(Client::builder().proxy(proxy).build()?)
        }
    }
}
