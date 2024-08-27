use std::sync::Arc;

use config::Config;
use log::info;
use runtime::Runtime;
use tokio::{select, signal};
use tokio_context::context::RefContext;

mod cmd_server;
mod command;
mod config;
mod db;
mod discover;
mod node;
mod protocol;
mod runtime;
mod until;

fn main() {
    // 启动在当前主线程
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { main0().await.unwrap() });
}

pub async fn main0() -> anyhow::Result<()> {
    env_logger::init();

    let cfg = Arc::new(Config::default());

    let rt = Runtime::new();

    let (ctx, ctx_handler) = RefContext::new();

    let (discover_handler, command_handler) = rt.start(&ctx, cfg.clone())?;

    shutdown_signal().await;

    ctx_handler.cancel();

    discover_handler.await?;
    command_handler.await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        info!("Got Ctrl+C signal shutdown program")
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        info!("Recv terminate signal shutdown program")
    };

    select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
