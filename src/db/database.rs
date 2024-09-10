use log::{debug, error, info};
use std::{collections::HashMap, sync::Arc};
use tokio::{select, sync::mpsc};
use tokio_context::context::{Context, RefContext};

use crate::{
    cluster::to_cluster,
    command::Command,
    connection::manager::ConnectionManager,
    node::ShareNodeTable,
    protocol::{frame::Frame, length::Length},
};

use super::dbvalue::DBValue;

pub struct Database {
    pub db: HashMap<String, DBValue>,
    pub tx: mpsc::Sender<Command>,
}

impl Database {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {
        Database {
            db: HashMap::new(),
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

pub async fn start_database_channel(
    ctx: RefContext,
    mut db: Database,
    mut db_recv: mpsc::Receiver<Command>,
    conn_manager: ConnectionManager,
) -> anyhow::Result<()> {
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
                        // 集群阶段
                        if let Err(err) = to_cluster(&conn_manager, &command).await {
                            error!("to cluster error: {:?}", err)
                        }
                    },
                    Err(err) => error!("Execute command error: {:?}", err)
                }
            }
        }
    }
    Ok(())
}
