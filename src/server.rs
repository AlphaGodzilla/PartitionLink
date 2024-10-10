use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::command::Command;
use command::{hash_get::HashMapGetCmd, hash_put::HashMapPutCmd};
use config::Config;
use db::{database::Database, dbvalue::DBValue};
use log::{debug, error, info};
use runtime::Runtime;
use tokio::{select, signal, sync::mpsc, task::JoinHandle};
use tokio_context::context::RefContext;

mod cluster;
mod cmd_server;
mod command;
mod config;
mod connection;
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

    let send_cmd = option_env!("LOCAL_CMD_MODE");
    if send_cmd.is_some() {
        info!("LOCAL_CMD_MODE set, running server and execute command mode");
        interval_execute_cmd(&runtime, db, tx.clone(), rx);
    } else {
        info!("LOCAL_CMD_MODE unset, running only server mode");
        // 阻塞当前线程
        runtime.block_on(async move { start_runtime(db, rx).await.unwrap() });
    }
}

pub fn interval_execute_cmd(
    runtime: &tokio::runtime::Runtime,
    db: Database,
    database_send: mpsc::Sender<Command>,
    database_recv: mpsc::Receiver<Command>,
) {
    // start server
    let server_handler =
        runtime.spawn(async move { start_runtime(db, database_recv).await.unwrap() });

    // try execute cmd
    execute_cmd(runtime, database_send, &server_handler);
}

pub fn execute_cmd(
    runtime: &tokio::runtime::Runtime,
    database_send: mpsc::Sender<Command>,
    server_handler: &JoinHandle<()>,
) {
    let adder = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        thread::sleep(Duration::from_secs(5));

        if server_handler.is_finished() {
            break;
        }

        // execute cmd
        let tx = database_send.clone();

        let adder_copy = adder.clone();
        runtime.spawn(async move {
            let (dbvalue_tx, mut dbvalue_rx) = mpsc::channel(1);
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
                match res {
                    Ok(res) => {
                        if let Some(v) = res {
                            debug!("Execute command HashMapPutCmd, got result => {}", &v);
                        } else {
                            debug!("Execute command HashMapPutCmd, got result => None");
                        }
                    }
                    Err(err) => {
                        debug!("Execute command HashMapPutCmd, got result => {:?}", &err);
                    }
                }
            }
            let cmd = Box::new(HashMapGetCmd {
                key: String::from("UserConnectStateMap"),
                member_key: String::from("jason"),
            });
            if let Err(err) = tx.send(Command::new(cmd, Some(dbvalue_tx.clone()))).await {
                error!("Send command error {:?}", err);
            }
            if let Some(res) = dbvalue_rx.recv().await {
                match res {
                    Ok(res) => {
                        if let Some(v) = res {
                            debug!("Execute command HashMapGetCmd, got result => {}", &v);
                        } else {
                            debug!("Execute command HashMapGetCmd, got result => None");
                        }
                    }
                    Err(err) => {
                        debug!("Execute command HashMapGetCmd, got result => {:?}", &err);
                    }
                }
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
