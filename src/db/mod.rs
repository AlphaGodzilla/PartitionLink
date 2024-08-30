use std::collections::HashMap;

use dbvalue::DBValue;
use log::{error, info};
use tokio::{select, sync::mpsc};
use tokio_context::context::{Context, RefContext};

use crate::command::Command;
pub mod dbvalue;

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
) -> anyhow::Result<()> {
    let (mut done_ctx, _handler) = Context::with_parent(&ctx, None);
    loop {
        select! {
            _ = done_ctx.done() => {
                info!("Database channel loop stop");
                break;
            },
            Some(cmd) = db_recv.recv() => {
                if let Err(err) = cmd.execute_and_send(&mut db).await {
                    error!("Execute command error: {:?}", err);
                }
            }
        }
    }
    Ok(())
}
