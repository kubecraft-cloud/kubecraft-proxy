use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    log::info!(target: "kubecraft-proxy", "starting up");

    // todo(iverly): add logic here

    log::info!(target: "kubecraft-proxy", "shutting down");
}
