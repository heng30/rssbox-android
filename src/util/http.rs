use crate::config;
use reqwest::{Client, Proxy, Result};

pub fn client() -> Result<Client> {
    let config = config::socks5();

    Ok(if config.enabled {
        let proxy = Proxy::all(format!("socks5://{}:{}", config.url, config.port))?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::new()
    })
}
