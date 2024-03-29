extern crate rssbox;

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() {
    rssbox::desktop_main().await;
}
