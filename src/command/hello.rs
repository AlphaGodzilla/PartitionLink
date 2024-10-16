use crate::db::{database::Database, dbvalue::DBValue};
use crate::runtime::Runtime;
use async_trait::async_trait;
use prost::Message;
use std::any::Any;
use std::fmt::Display;

use super::{
    proto::{ProtoCmd, ProtoCommand, ProtoHelloCmd},
    CommandType, ExecutableCommand,
};

#[derive(Clone)]
pub struct HelloCmd {
    pub valid: bool,
}

#[async_trait]
impl ExecutableCommand for HelloCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::READ
    }

    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoHelloCmd { valid: self.valid };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> ProtoCmd {
        ProtoCmd::HelloCmd
    }
    fn as_any(&self) -> &dyn Any {
        self
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
