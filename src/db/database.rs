use std::sync::Arc;
use ahash::AHashMap;
use log::{debug, error, info, log_enabled};
use log::Level::Debug;
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_context::context::{Context, RefContext};

use super::dbvalue::DBValue;
use crate::postman::PostMessage;
use crate::{
    cluster::{self},
    command::Command,
    connection::manager::ConnectionManager,
};
use crate::runtime::Runtime;

pub struct Database {
    pub db: AHashMap<String, DBValue>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            db: AHashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: DBValue) {
        self.db.insert(key, value);
    }

    pub fn get(&mut self, key: &str) -> Option<&DBValue> {
        self.db.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut DBValue> {
        self.db.get_mut(key)
    }
}

pub fn start_db_cmd_channel(
    app: Arc<Runtime>,
    ctx: RefContext,
    mut db: Database,
    mut db_recv: mpsc::Receiver<Box<dyn PostMessage>>,
) -> anyhow::Result<JoinHandle<()>> {
    let hander = tokio::spawn(async move {
        info!("Database channel thread startup");
        let (mut done_ctx, _handler) = Context::with_parent(&ctx, None);
        loop {
            select! {
                _ = done_ctx.done() => {
                    info!("Database channel loop stop");
                    break;
                },
                Some(command) = db_recv.recv() => {
                    if let Some(command ) = command.as_any().downcast_ref::<Command>() {
                        match command.execute_and_send(Some(app.as_ref()), Some(&mut db)).await  {
                            Ok(_) => {
                                // // 集群广播
                                // if let Err(err) = cluster::broadcast(&conn_manager, &command).await {
                                //     error!("to cluster error: {:?}", err)
                                // }
                                if log_enabled!(Debug) {
                                    debug!("命令执行成功: {}", command.inner_ref())
                                }
                            },
                            Err(err) => error!("Execute command error: {:?}", err)
                        }
                    }
                }
            }
        }
    });
    Ok(hander)
}
