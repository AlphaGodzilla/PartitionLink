use crate::db::{database::Database, dbvalue::DBValue};
use crate::runtime::Runtime;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Display;

use super::{proto::ProtoCmd, CommandType, ExecutableCommand};

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

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        Err(anyhow::anyhow!("InvalidCommand cannot encode"))
    }

    fn cmd_id(&self) -> ProtoCmd {
        ProtoCmd::Unknown
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
