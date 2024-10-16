use crate::command::proto::ProtoCmd;
use crate::db::{database::Database, dbvalue::DBValue};
use crate::postman::{Channel, LetterMessage};
use crate::protocol::frame::{self, Frame};
use crate::protocol::kind::Kind;
use crate::runtime::Runtime;
use crate::until;
use async_trait::async_trait;
use hello::HelloCmd;
use invalid::InvalidCommand;
use prost::Message;
use prost_types::Timestamp;
use proto::ProtoCommand;
use register_info::parse_proto_command;
use std::any::Any;
use std::fmt::Display;
use tokio::sync::mpsc;

pub mod hash_get;
pub mod hash_put;
pub mod hello;
pub mod invalid;
pub mod raft;
pub mod register_info;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum CommandType {
    READ,
    WRITE,
}

// 所有命令必须实现该trait
#[async_trait]
pub trait ExecutableCommand: Display + Send + Sync {
    // 命令类型，分为读类型和写类型
    fn cmd_type(&self) -> CommandType;

    // 执行命令
    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>>;

    // 编码为字节数组
    fn encode(&self) -> anyhow::Result<bytes::Bytes>;

    // 编码的命令ID
    fn cmd_id(&self) -> ProtoCmd;
    fn as_any(&self) -> &dyn Any;

    fn is_raft_cmd(&self) -> bool {
        self.cmd_id() == ProtoCmd::RaftCmd
    }

    fn is_write_type(&self) -> bool {
        self.cmd_type() == CommandType::WRITE
    }

    fn is_read_type(&self) -> bool {
        self.cmd_type() == CommandType::READ
    }

    fn is_valid(&self) -> bool {
        self.cmd_id() != ProtoCmd::Unknown
    }
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

    pub async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        self.inner.execute(app, db).await
    }

    pub async fn send(&self, value: anyhow::Result<Option<DBValue>>) -> anyhow::Result<()> {
        match &self.tx {
            Some(tx) => tx.send(value).await?,
            _ => {}
        }
        Ok(())
    }

    pub async fn execute_and_send(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<()> {
        self.send(self.execute(app, db).await).await?;
        Ok(())
    }

    pub fn encode_to_payload(&self) -> anyhow::Result<bytes::Bytes> {
        let now_ts = until::now_ts()? as i64;
        let now_ts_sec: i64 = now_ts / 1000;
        let now_ts_mills: i64 = now_ts - now_ts_sec * 1000;
        let now_ts_nanos: i64 = now_ts_mills * 1000000;

        let ts = Timestamp {
            seconds: now_ts_sec,
            nanos: now_ts_nanos as i32,
        };
        let msg = ProtoCommand {
            cmd: self.inner.cmd_id() as i32,
            ts: Some(ts),
            value: self.inner.encode()?.to_vec(),
        };
        let mut buff = bytes::BytesMut::new();
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    pub fn encode_to_frames(&self) -> anyhow::Result<Vec<Frame>> {
        let payload = self.encode_to_payload()?;
        frame::build_frames(Kind::CMD, &payload[..])
    }

    pub fn inner_ref(&self) -> &Box<dyn ExecutableCommand> {
        &self.inner
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
                    Command::new(cmd, None)
                } else {
                    Command::new(Box::new(InvalidCommand {}), None)
                }
            }
            Err(_) => Command::new(Box::new(InvalidCommand {}), None),
        }
    }
}

impl LetterMessage for Command {
    fn channel(&self) -> Channel {
        if self.inner_ref().is_raft_cmd() {
            Channel::RaftMsg
        } else {
            Channel::DbCmdReq
        }
    }
}

pub struct ProposalCommand(pub Command);

impl LetterMessage for ProposalCommand {
    fn channel(&self) -> Channel {
        Channel::RaftProposal
    }
}
