use crate::db::{DBValue, Database};

use super::{CommandType, ExecutableCommand};

#[derive(Debug, Clone)]
pub struct InvalidCommand {}

impl ExecutableCommand for InvalidCommand {
    fn cmd_type(&self) -> CommandType {
        CommandType::READ
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }
}
