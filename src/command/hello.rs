use std::fmt::Display;

use prost::Message;

use crate::db::{dbvalue::DBValue, Database};

use super::{CommandType, ExecutableCommand};

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
        let msg = crate::command::proto::out::HelloCmd { valid: self.valid };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }
}

impl Display for HelloCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hello:{}", self.valid)
    }
}

impl From<super::proto::out::HelloCmd> for HelloCmd {
    fn from(value: super::proto::out::HelloCmd) -> Self {
        HelloCmd { valid: value.valid }
    }
}
