use std::{fmt::Display, time::SystemTime};

use hello::HelloCmd;
use invalid::InvalidCommand;
use prost::Message;
use prost_types::Timestamp;
use tokio::sync::mpsc;

use crate::{
    db::{dbvalue::DBValue, Database},
    until,
};

pub mod hash_get;
pub mod hash_put;
pub mod hello;
pub mod invalid;
pub mod proto;

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

    pub fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        let now_ts = until::now_ts()? as i64;
        let now_ts_sec: i64 = now_ts / 1000;
        let now_ts_mills: i64 = now_ts - now_ts_sec * 1000;
        let now_ts_nanos: i64 = now_ts_mills * 1000000;
        let ts = Timestamp {
            seconds: now_ts_sec,
            nanos: now_ts_nanos as i32,
        };
        let msg = crate::command::proto::out::Command {
            cmd: crate::command::proto::out::Cmd::HelloCmd as i32,
            ts: Some(ts),
            value: self.inner.encode()?.to_vec(),
        };
        let mut buff = bytes::BytesMut::new();
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
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
