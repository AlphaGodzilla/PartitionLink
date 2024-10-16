use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::command::Command;
use crate::proto::{HashGetCmd, HashPutCmd};
use db::dbvalue::DBValue;
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
mod postman;
mod proto;
mod protocol;
mod runtime;
mod until;

fn main() {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let send_cmd = option_env!("LOCAL_CMD_MODE");
    if send_cmd.is_some() {
        info!("LOCAL_CMD_MODE set, running server and execute command mode");
        interval_execute_cmd(&runtime);
    } else {
        info!("LOCAL_CMD_MODE unset, running only server mode");
        // 阻塞当前线程
        let app = Arc::new(Runtime::new_with_default_config());
        let app_ref = app.clone();
        runtime.block_on(async move { start_runtime(app_ref).await.unwrap() });
    }
}

pub fn interval_execute_cmd(runtime: &tokio::runtime::Runtime) {
    let app = Arc::new(Runtime::new_with_default_config());
    let app_ref = app.clone();
    // start server
    let server_handler = runtime.spawn(async move { start_runtime(app_ref).await.unwrap() });

    // try execute cmd
    execute_cmd(runtime, app.clone(), &server_handler);
}

pub fn execute_cmd(
    runtime: &tokio::runtime::Runtime,
    app: Arc<Runtime>,
    server_handler: &JoinHandle<()>,
) {
    let adder = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        thread::sleep(Duration::from_secs(5));

        if server_handler.is_finished() {
            break;
        }

        let adder_copy = adder.clone();
        let app_ref = app.clone();
        runtime.spawn(async move {
            let (dbvalue_tx, mut dbvalue_rx) = mpsc::channel(1);
            let cmd = Box::new(HashPutCmd {
                key: String::from("UserConnectStateMap"),
                member_key: String::from("jason"),
                member_value: Some(
                    DBValue::String(String::from(format!(
                        "online: {}",
                        adder_copy.fetch_add(1, Ordering::SeqCst)
                    )))
                    .into(),
                ),
            });
            let command = Box::new(Command::new(cmd, Some(dbvalue_tx.clone())));
            if let Err(err) = app_ref.postman.send(command).await {
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
            let cmd = Box::new(HashGetCmd {
                key: String::from("UserConnectStateMap"),
                member_key: String::from("jason"),
            });
            let command = Box::new(Command::new(cmd, Some(dbvalue_tx.clone())));
            if let Err(err) = app_ref.postman.send(command).await {
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

pub async fn start_runtime(app: Arc<Runtime>) -> anyhow::Result<()> {
    let (ctx, ctx_handler) = RefContext::new();

    let mut handlers = Runtime::start(app, ctx).await?;

    shutdown_signal().await;

    ctx_handler.cancel();

    for handler in handlers.drain(..) {
        handler.await?;
    }
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
