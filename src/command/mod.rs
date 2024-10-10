use std::fmt::Display;
use log::trace;

use hello::HelloCmd;
use invalid::InvalidCommand;
use prost::Message;
use prost_types::Timestamp;
use proto::ProtoCommand;
use register_info::parse_proto_command;
use tokio::sync::mpsc;

use crate::db::{database::Database, dbvalue::DBValue};
use crate::protocol::frame::Frame;
use crate::protocol::kind::Kind;
use crate::protocol::length::Length;
use crate::protocol::MAX_PAYLOAD_LENGTH;

pub mod hash_get;
pub mod hash_put;
pub mod hello;
pub mod invalid;
pub mod proto;
pub mod register_info;

pub enum CommandType {
    READ,
    WRITE,
}

// 所有命令必须实现该trait
pub trait ExecutableCommand: Display + Send + Sync {
    // 命令类型，分为读类型和写类型
    fn cmd_type(&self) -> CommandType;

    // 执行命令
    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>>;

    // 编码为字节数组
    fn encode(&self) -> anyhow::Result<bytes::Bytes>;

    // 编码的命令ID
    fn cmd_id(&self) -> i32;
}

pub struct Command {
    // 命令
    inner: Box<dyn ExecutableCommand>,
    // 用于返回命令执行结果的发送器
    tx: Option<mpsc::Sender<anyhow::Result<Option<DBValue>>>>,
}

impl Command {
    pub fn new(
        impl_cmd: Box<dyn ExecutableCommand>,
        tx: Option<mpsc::Sender<anyhow::Result<Option<DBValue>>>>,
    ) -> Self {
        Command {
            inner: impl_cmd,
            tx,
        }
    }

    pub fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        self.inner.execute(db)
    }

    pub async fn send(&self, value: anyhow::Result<Option<DBValue>>) -> anyhow::Result<()> {
        match &self.tx {
            Some(tx) => tx.send(value).await?,
            _ => {}
        }
        Ok(())
    }

    pub async fn execute_and_send(&self, db: &mut Database) -> anyhow::Result<()> {
        self.send(self.execute(db)).await?;
        Ok(())
    }

    pub fn encode_to_payload(&self) -> anyhow::Result<bytes::Bytes> {
        // let now_ts = until::now_ts()? as i64;
        // let now_ts_sec: i64 = now_ts / 1000;
        // let now_ts_mills: i64 = now_ts - now_ts_sec * 1000;
        // let now_ts_nanos: i64 = now_ts_mills * 1000000;
        let now_ts = 0;
        let now_ts_sec: i64 = 0;
        let now_ts_mills: i64 = 0;
        let now_ts_nanos: i64 = 0;

        let ts = Timestamp {
            seconds: now_ts_sec,
            nanos: now_ts_nanos as i32,
        };
        let msg = ProtoCommand {
            cmd: self.inner.cmd_id(),
            ts: Some(ts),
            value: self.inner.encode()?.to_vec(),
        };
        let mut buff = bytes::BytesMut::new();
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    pub fn encode_to_frames(&self) -> anyhow::Result<Vec<Frame>> {
        let payload = self.encode_to_payload()?;
        let mut chunks = payload.chunks(MAX_PAYLOAD_LENGTH as usize).peekable();
        let chunks_size = chunks.len();
        let mut current_chunk = 0;
        let mut frames = Vec::new();
        loop {
            if chunks.peek().is_none() {
                trace!("command chunk is none, break loop");
                break;
            }
            if let Some(chunk) = chunks.next() {
                current_chunk += 1;
                trace!("next chunk current chunk {}", current_chunk);
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
                    .set_kind(Kind::CMD)
                    .set_length(Length::new(chunk.len() as u8))
                    .set_payload(payload);
                frames.push(frame);
            }
        }
        Ok(frames)
    }

}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "hello" => Command::new(Box::new(HelloCmd { valid: true }), None),
            _ => Command::new(Box::new(InvalidCommand {}), None),
        }
    }
}

// 从字节数组解析Command
impl From<&[u8]> for Command {
    fn from(value: &[u8]) -> Self {
        match ProtoCommand::decode(value) {
            Ok(command) => {
                if let Ok(cmd) = parse_proto_command(command) {
                    return Command::new(cmd, None);
                } else {
                    return Command::new(Box::new(InvalidCommand {}), None);
                }
            }
            Err(_) => Command::new(Box::new(InvalidCommand {}), None),
        }
    }
}
