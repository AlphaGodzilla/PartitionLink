use std::fmt::{Display, Formatter};

use bytes::Bytes;

use prost::Message;

use crate::command::proto::ProtoRaftCmd;
use crate::command::{CommandType, ExecutableCommand};
use crate::db::database::Database;
use crate::db::dbvalue::DBValue;

use super::proto::{ProtoCmd, ProtoCommand};

#[derive(Clone)]
pub struct RaftCmd {
    pub body: DBValue,
}

impl ExecutableCommand for RaftCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::WRITE
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        todo!()
    }

    fn encode(&self) -> anyhow::Result<Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoRaftCmd {
            body: Some(self.body.to_protobuf().into()),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> i32 {
        ProtoCmd::RaftCmd as i32
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
