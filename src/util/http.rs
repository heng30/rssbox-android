use crate::config;
use reqwest::{Client, Proxy, Result};

pub enum ProxyType {
    Http,
    Socks5,
}

pub fn convert_to_type(pt: &str) -> ProxyType {
    match pt.to_lowercase().as_str() {
        "http" => ProxyType::Http,
        _ => ProxyType::Socks5,
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
                ProxyType::Socks5 => {
                    Proxy::all(format!("socks5://{}:{}", config.socks5_url, config.http_port))?
                }
            };
            Ok(Client::builder().proxy(proxy).build()?)
        }
    }
}
