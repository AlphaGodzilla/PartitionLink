use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::command::Command;
use command::hashmap::{HashMapGetCmd, HashMapPutCmd};
use config::Config;
use db::{DBValue, Database};
use log::{debug, error, info};
use runtime::Runtime;
use tokio::{select, signal, sync::mpsc};
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
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // make database channel size of 32
    let (tx, rx) = mpsc::channel(32);
    let db = Database::new(tx.clone());

    // start server
    runtime.spawn(async move { start_runtime(db, rx).await.unwrap() });

    // try execute cmd
    let adder = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        thread::sleep(Duration::from_secs(1));

        // execute cmd
        let tx = tx.clone();

        let adder_copy = adder.clone();
        runtime.spawn(async move {
            let (dbvalue_tx, mut dbvalue_rx) = mpsc::channel(1);
            // let cmd = Box::new(HelloCmd { valid: true });
            let cmd = Box::new(HashMapPutCmd {
                key: String::from("UserConnectStateMap"),
                member_key: String::from("jason"),
                member_value: DBValue::String(String::from(format!(
                    "online: {}",
                    adder_copy.fetch_add(1, Ordering::SeqCst)
                ))),
            });
            if let Err(err) = tx.send(Command::new(cmd, Some(dbvalue_tx.clone()))).await {
                error!("Send command error {:?}", err);
            }
            if let Some(res) = dbvalue_rx.recv().await {
                debug!("Execute command HashMapPutCmd, got result => {:?}", &res);
            }
            let cmd = Box::new(HashMapGetCmd {
                key: String::from("UserConnectStateMap"),
                member_key: String::from("jason"),
            });
            if let Err(err) = tx.send(Command::new(cmd, Some(dbvalue_tx.clone()))).await {
                error!("Send command error {:?}", err);
            }
            if let Some(res) = dbvalue_rx.recv().await {
                debug!("Execute command HashMapGetCmd, got result => {:?}", &res);
            }
        });
    }
}

pub async fn start_runtime(
    database: Database,
    database_recv: mpsc::Receiver<Command>,
) -> anyhow::Result<()> {
    let cfg = Arc::new(Config::default());

    let rt = Runtime::new();

    let (ctx, ctx_handler) = RefContext::new();

    let (discover_handler, command_handler, db_channel_handler) =
        rt.start(&ctx, cfg.clone(), database, database_recv)?;

    shutdown_signal().await;

    ctx_handler.cancel();

    discover_handler.await?;
    command_handler.await?;
    db_channel_handler.await?;
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
