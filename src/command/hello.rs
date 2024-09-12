use std::fmt::Display;

use prost::Message;

use crate::db::{database::Database, dbvalue::DBValue};

use super::{
    proto::{ProtoCmd, ProtoCommand, ProtoHelloCmd},
    CommandType, ExecutableCommand,
};

#[derive(Clone)]
pub struct HelloCmd {
    pub valid: bool,
}

impl ExecutableCommand for HelloCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::READ
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoHelloCmd { valid: self.valid };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> i32 {
        ProtoCmd::HelloCmd as i32
    }
}

impl Display for HelloCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hello:{}", self.valid)
    }
}

impl From<ProtoHelloCmd> for HelloCmd {
    fn from(value: ProtoHelloCmd) -> Self {
        HelloCmd { valid: value.valid }
    }
}

impl TryFrom<ProtoCommand> for HelloCmd {
    type Error = anyhow::Error;

    fn try_from(value: ProtoCommand) -> Result<Self, Self::Error> {
        let hello_cmd = ProtoHelloCmd::decode(&value.value[..])?;
        Ok(hello_cmd.into())
    }
}
