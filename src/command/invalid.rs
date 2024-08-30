use std::fmt::Display;

use crate::db::{dbvalue::DBValue, Database};

use super::{CommandType, ExecutableCommand};

#[derive(Clone)]
pub struct InvalidCommand {}

impl ExecutableCommand for InvalidCommand {
    fn cmd_type(&self) -> CommandType {
        CommandType::READ
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        Err(anyhow::anyhow!("InvalidCommand cannot encode"))
    }
}

impl Display for InvalidCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid",)
    }
}
