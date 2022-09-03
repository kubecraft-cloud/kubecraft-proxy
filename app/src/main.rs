use anyhow::Result;
use std::env;

use proxy::Proxy;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    log::info!(target: "kubecraft-proxy", "starting up");

    let proxy = Proxy::new();
    proxy.start().await?;

    log::info!(target: "kubecraft-proxy", "shutting down");
    Ok(())
}
