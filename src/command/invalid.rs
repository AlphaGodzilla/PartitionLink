use super::{CommandType, ExecutableCommand};
use crate::db::{database::Database, dbvalue::DBValue};
use crate::proto::command_message::Cmd;
use crate::runtime::Runtime;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Display;

#[derive(Clone)]
pub struct InvalidCommand {}

#[async_trait]
impl ExecutableCommand for InvalidCommand {
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
        Err(anyhow::anyhow!("InvalidCommand cannot to cmd"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Display for InvalidCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid",)
    }
}
