use super::{CommandType, ExecutableCommand};
use crate::db::{database::Database, dbvalue::DBValue};
use crate::proto::command_message::Cmd;
use crate::proto::HelloCmd;
use crate::runtime::Runtime;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Display;

// #[derive(Clone)]
// pub struct HelloCmd {
//     pub valid: bool,
// }

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

    fn to_cmd(&self) -> anyhow::Result<Cmd> {
        Ok(Cmd::Hello(self.clone()))
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

impl TryFrom<Cmd> for HelloCmd {
    type Error = anyhow::Error;

    fn try_from(value: Cmd) -> Result<Self, Self::Error> {
        if let Cmd::Hello(hello_cmd) = value {
            Ok(hello_cmd)
        } else {
            Err(anyhow::anyhow!("invalid command"))
        }
    }
}
