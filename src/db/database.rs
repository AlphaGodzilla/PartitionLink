use ahash::AHashMap;
use log::{error, info};
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_context::context::{Context, RefContext};

use crate::{
    cluster::{self},
    command::Command,
    connection::manager::ConnectionManager,
};

use super::dbvalue::DBValue;

pub struct Database {
    pub db: AHashMap<String, DBValue>,
    pub tx: mpsc::Sender<Command>,
}

impl Database {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {
        Database {
            db: AHashMap::new(),
            tx,
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
    ctx: RefContext,
    mut db: Database,
    mut db_recv: mpsc::Receiver<Command>,
    conn_manager: ConnectionManager,
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
                    match command.execute_and_send(&mut db).await  {
                        Ok(_) => {
                            // 集群广播
                            if let Err(err) = cluster::broadcast(&conn_manager, &command).await {
                                error!("to cluster error: {:?}", err)
                            }
                        },
                        Err(err) => error!("Execute command error: {:?}", err)
                    }
                }
            }
        }
    });
    Ok(hander)
}
