use std::fmt::Debug;

use hello::HelloCmd;
use invalid::InvalidCommand;
use tokio::sync::mpsc;

use crate::db::{DBValue, Database};

pub mod hashmap;
pub mod hello;
pub mod invalid;
pub mod proto;

pub enum CommandType {
    READ,
    WRITE,
}

// 所有命令必须实现该trait
pub trait ExecutableCommand: Debug + Send + Sync {
    // 命令类型，分为读类型和写类型
    fn cmd_type(&self) -> CommandType;

    // 执行命令
    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>>;
}

#[derive(Debug)]
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

    pub async fn execute_and_send(&self, db: &mut Database) -> anyhow::Result<()> {
        match &self.tx {
            Some(tx) => tx.send(self.execute(db)).await?,
            _ => {}
        }
        Ok(())
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
