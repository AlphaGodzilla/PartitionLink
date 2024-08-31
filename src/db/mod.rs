use std::collections::HashMap;

use dbvalue::DBValue;
use log::{debug, error, info};
use tokio::{select, sync::mpsc};
use tokio_context::context::{Context, RefContext};

use crate::{
    command::Command,
    protocol::{frame::Frame, length::Length},
};
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
            Some(command) = db_recv.recv() => {
                match command.execute_and_send(&mut db).await  {
                    Ok(_) => {
                        // 执行完命令后的传播命令阶段
                        encode_to_frame_and_send(&command)?;
                    },
                    Err(err) => error!("Execute command error: {:?}", err)
                }
            }
        }
    }
    Ok(())
}

fn encode_to_frame_and_send(command: &Command) -> anyhow::Result<()> {
    let payload = command.encode()?;
    let mut chunks = payload.chunks(256).peekable();
    let chunks_size = chunks.len();
    let mut current_chunk = 0;
    loop {
        if chunks.peek().is_none() {
            break;
        }
        if let Some(chunk) = chunks.next() {
            current_chunk += 1;
            let is_last = current_chunk == chunks_size;
            let frame_head;
            if is_last {
                frame_head = crate::protocol::head::Head::FIN;
            } else {
                frame_head = crate::protocol::head::Head::UNFIN;
            }
            let mut frame = Frame::new();
            let mut payload = Vec::with_capacity(chunk.len());
            payload.extend_from_slice(chunk);
            frame
                .set_head(frame_head)
                .set_length(Length::new(chunk.len() as u8))
                .set_payload(payload);
            // TODO: next step save to raft log and broadcast frame to other node
            debug!("save frame to raft log and broadcast frame to other node");
        }
    }

    Ok(())
}
