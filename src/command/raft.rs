use crate::command::proto::ProtoRaftCmd;
use crate::command::{CommandType, ExecutableCommand};
use crate::db::database::Database;
use crate::db::dbvalue::DBValue;
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use prost::Message as PrMessage;
use std::any::Any;
use std::fmt::{Display, Formatter};

use super::proto::{ProtoCmd, ProtoCommand};

use crate::postman::Channel;
use crate::postman::PostMessage;
use crate::runtime::Runtime;
use protobuf::Message as PbMessage;
use raft::prelude::Message;

#[derive(Clone)]
pub struct RaftCmd {
    pub body: DBValue,
}

#[async_trait]
impl ExecutableCommand for RaftCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::WRITE
    }

    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }

    fn encode(&self) -> anyhow::Result<Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoRaftCmd {
            body: Some(self.body.to_protobuf().into()),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> ProtoCmd {
        ProtoCmd::RaftCmd
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Display for RaftCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RaftCmd")
    }
}

impl From<ProtoRaftCmd> for RaftCmd {
    fn from(value: ProtoRaftCmd) -> Self {
        RaftCmd {
            body: value.body.map_or(DBValue::None, |x| x.into()),
        }
    }
}

impl TryFrom<ProtoCommand> for RaftCmd {
    type Error = anyhow::Error;

    fn try_from(value: ProtoCommand) -> Result<Self, Self::Error> {
        let cmd = ProtoRaftCmd::decode(&value.value[..])?;
        Ok(cmd.into())
    }
}

impl RaftCmd {
    pub fn to_raft_message(&self) -> anyhow::Result<raft::prelude::Message> {
        match &self.body {
            DBValue::Bytes(bytes) => Ok(Message::parse_from_bytes(&bytes[..])?),
            _ => Err(anyhow!("value not bytes type")),
        }
    }
}
